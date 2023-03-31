use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
#[cfg(feature = "mock_long_computations")]
use std::time::Duration;

use actix_identity::Identity;
use actix_web::rt::spawn;
use actix_web::rt::task::spawn_blocking;
use actix_web::rt::time::timeout;
use actix_web::{get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use adf_bdd::datatypes::adf::VarContainer;
use adf_bdd::datatypes::{BddNode, Term, Var};
use futures_util::FutureExt;
use mongodb::bson::doc;
use mongodb::bson::{to_bson, Bson};
use names::{Generator, Name};
use serde::{Deserialize, Serialize};

use adf_bdd::adf::{Adf, DoubleLabeledGraph};
use adf_bdd::adfbiodivine::Adf as BdAdf;
use adf_bdd::parser::AdfParser;

use crate::config::{AppState, RunningInfo, Task, ADF_COLL, COMPUTE_TIME, DB_NAME, USER_COLL};
use crate::user::{username_exists, User};

type Ac = Vec<Term>;
type AcDb = Vec<String>;

#[derive(Copy, Clone, Deserialize, Serialize)]
pub(crate) enum Parsing {
    Naive,
    Hybrid,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub(crate) enum Strategy {
    Ground,
    Complete,
    Stable,
    StableCountingA,
    StableCountingB,
    StableNogood,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct AcAndGraph {
    pub(crate) ac: AcDb,
    pub(crate) graph: DoubleLabeledGraph,
}

impl From<AcAndGraph> for Bson {
    fn from(source: AcAndGraph) -> Self {
        to_bson(&source).expect("Serialization should work")
    }
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) enum OptionWithError<T> {
    Some(T),
    Error(String),
    #[default]
    None,
}

impl<T> OptionWithError<T> {
    fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }
}

impl<T: Serialize> From<OptionWithError<T>> for Bson {
    fn from(source: OptionWithError<T>) -> Self {
        to_bson(&source).expect("Serialization should work")
    }
}

type AcsAndGraphsOpt = OptionWithError<Vec<AcAndGraph>>;

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct AcsPerStrategy {
    pub(crate) parse_only: AcsAndGraphsOpt,
    pub(crate) ground: AcsAndGraphsOpt,
    pub(crate) complete: AcsAndGraphsOpt,
    pub(crate) stable: AcsAndGraphsOpt,
    pub(crate) stable_counting_a: AcsAndGraphsOpt,
    pub(crate) stable_counting_b: AcsAndGraphsOpt,
    pub(crate) stable_nogood: AcsAndGraphsOpt,
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct VarContainerDb {
    names: Vec<String>,
    mapping: HashMap<String, String>,
}

impl From<VarContainer> for VarContainerDb {
    fn from(source: VarContainer) -> Self {
        Self {
            names: source.names().read().unwrap().clone(),
            mapping: source
                .mappings()
                .read()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        }
    }
}

impl From<VarContainerDb> for VarContainer {
    fn from(source: VarContainerDb) -> Self {
        Self::from_parser(
            Arc::new(RwLock::new(source.names)),
            Arc::new(RwLock::new(
                source
                    .mapping
                    .into_iter()
                    .map(|(k, v)| (k, v.parse().unwrap()))
                    .collect(),
            )),
        )
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct BddNodeDb {
    var: String,
    lo: String,
    hi: String,
}

impl From<BddNode> for BddNodeDb {
    fn from(source: BddNode) -> Self {
        Self {
            var: source.var().0.to_string(),
            lo: source.lo().0.to_string(),
            hi: source.hi().0.to_string(),
        }
    }
}

impl From<BddNodeDb> for BddNode {
    fn from(source: BddNodeDb) -> Self {
        Self::new(
            Var(source.var.parse().unwrap()),
            Term(source.lo.parse().unwrap()),
            Term(source.hi.parse().unwrap()),
        )
    }
}

type SimplifiedBdd = Vec<BddNodeDb>;

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct SimplifiedAdf {
    pub(crate) ordering: VarContainerDb,
    pub(crate) bdd: SimplifiedBdd,
    pub(crate) ac: AcDb,
}

impl SimplifiedAdf {
    fn from_lib_adf(adf: Adf) -> Self {
        SimplifiedAdf {
            ordering: adf.ordering.into(),
            bdd: adf.bdd.nodes.into_iter().map(Into::into).collect(),
            ac: adf.ac.into_iter().map(|t| t.0.to_string()).collect(),
        }
    }
}

type SimplifiedAdfOpt = OptionWithError<SimplifiedAdf>;

#[derive(Deserialize, Serialize)]
pub(crate) struct AdfProblem {
    pub(crate) name: String,
    pub(crate) username: String,
    pub(crate) code: String,
    pub(crate) parsing_used: Parsing,
    pub(crate) adf: SimplifiedAdfOpt,
    pub(crate) acs_per_strategy: AcsPerStrategy,
}

#[derive(Clone, Deserialize)]
struct AddAdfProblemBody {
    name: Option<String>,
    code: String,
    parse_strategy: Parsing,
}

async fn adf_problem_exists(
    adf_coll: &mongodb::Collection<AdfProblem>,
    name: &str,
    username: &str,
) -> bool {
    adf_coll
        .find_one(doc! { "name": name, "username": username }, None)
        .await
        .ok()
        .flatten()
        .is_some()
}

#[derive(Serialize)]
struct AdfProblemInfo {
    name: String,
    code: String,
    parsing_used: Parsing,
    acs_per_strategy: AcsPerStrategy,
    running_tasks: Vec<Task>,
}

impl AdfProblemInfo {
    fn from_adf_prob_and_tasks(adf: AdfProblem, tasks: &HashSet<RunningInfo>) -> Self {
        AdfProblemInfo {
            name: adf.name.clone(),
            code: adf.code,
            parsing_used: adf.parsing_used,
            acs_per_strategy: adf.acs_per_strategy,
            running_tasks: tasks
                .iter()
                .filter_map(|t| {
                    (t.adf_name == adf.name && t.username == adf.username).then_some(t.task)
                })
                .collect(),
        }
    }
}

#[post("/add")]
async fn add_adf_problem(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
    req_body: web::Json<AddAdfProblemBody>,
) -> impl Responder {
    let adf_problem_input: AddAdfProblemBody = req_body.into_inner();
    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);
    let user_coll: mongodb::Collection<User> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(USER_COLL);

    let username = match identity.map(|id| id.id()) {
        None => {
            // Create and log in temporary user
            let gen = Generator::with_naming(Name::Numbered);
            let candidates = gen.take(10);

            let mut name: Option<String> = None;
            for candidate in candidates {
                if name.is_some() {
                    continue;
                }

                if !(username_exists(&user_coll, &candidate).await) {
                    name = Some(candidate);
                }
            }

            let username = match name {
                Some(name) => name,
                None => {
                    return HttpResponse::InternalServerError().body("Could not generate new name.")
                }
            };

            match user_coll
                .insert_one(
                    User {
                        username: username.clone(),
                        password: None,
                    },
                    None,
                )
                .await
            {
                Ok(_) => (),
                Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
            }

            Identity::login(&req.extensions(), username.clone()).unwrap();

            username
        }
        Some(Err(err)) => return HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => username,
    };

    let problem_name = match &adf_problem_input.name {
        Some(name) => {
            if adf_problem_exists(&adf_coll, name, &username).await {
                return HttpResponse::Conflict()
                    .body("ADF Problem with that name already exists. Please pick another one!");
            }

            name.clone()
        }
        None => {
            let gen = Generator::with_naming(Name::Numbered);
            let candidates = gen.take(10);

            let mut name: Option<String> = None;
            for candidate in candidates {
                if name.is_some() {
                    continue;
                }

                if !(adf_problem_exists(&adf_coll, &candidate, &username).await) {
                    name = Some(candidate);
                }
            }

            match name {
                Some(name) => name,
                None => {
                    return HttpResponse::InternalServerError().body("Could not generate new name.")
                }
            }
        }
    };

    let adf_problem: AdfProblem = AdfProblem {
        name: problem_name.clone(),
        username: username.clone(),
        code: adf_problem_input.code.clone(),
        parsing_used: adf_problem_input.parse_strategy,
        adf: SimplifiedAdfOpt::None,
        acs_per_strategy: AcsPerStrategy::default(),
    };

    let result = adf_coll.insert_one(&adf_problem, None).await;

    if let Err(err) = result {
        return HttpResponse::InternalServerError()
            .body(format!("Could not create Database entry. Error: {err}"));
    }

    let username_clone = username.clone();
    let problem_name_clone = problem_name.clone();

    let adf_fut = timeout(
        COMPUTE_TIME,
        spawn_blocking(move || {
            let running_info = RunningInfo {
                username: username_clone,
                adf_name: problem_name_clone,
                task: Task::Parse,
            };

            app_state
                .currently_running
                .lock()
                .unwrap()
                .insert(running_info.clone());

            #[cfg(feature = "mock_long_computations")]
            std::thread::sleep(Duration::from_secs(20));

            let parser = AdfParser::default();
            parser.parse()(&adf_problem_input.code)
                .map_err(|_| "ADF could not be parsed, double check your input!")?;

            let lib_adf = match adf_problem_input.parse_strategy {
                Parsing::Naive => Adf::from_parser(&parser),
                Parsing::Hybrid => {
                    let bd_adf = BdAdf::from_parser(&parser);
                    bd_adf.hybrid_step_opt(false)
                }
            };

            app_state
                .currently_running
                .lock()
                .unwrap()
                .remove(&running_info);

            let ac_and_graph = AcAndGraph {
                ac: lib_adf.ac.iter().map(|t| t.0.to_string()).collect(),
                graph: lib_adf.into_double_labeled_graph(None),
            };

            Ok::<_, &str>((SimplifiedAdf::from_lib_adf(lib_adf), ac_and_graph))
        }),
    );

    spawn(adf_fut.then(move |adf_res| async move {
        let (adf, ac_and_graph): (SimplifiedAdfOpt, AcsAndGraphsOpt) = match adf_res {
            Err(err) => (
                SimplifiedAdfOpt::Error(err.to_string()),
                AcsAndGraphsOpt::Error(err.to_string()),
            ),
            Ok(Err(err)) => (
                SimplifiedAdfOpt::Error(err.to_string()),
                AcsAndGraphsOpt::Error(err.to_string()),
            ),
            Ok(Ok(Err(err))) => (
                SimplifiedAdfOpt::Error(err.to_string()),
                AcsAndGraphsOpt::Error(err.to_string()),
            ),
            Ok(Ok(Ok((adf, ac_and_graph)))) => (
                SimplifiedAdfOpt::Some(adf),
                AcsAndGraphsOpt::Some(vec![ac_and_graph]),
            ),
        };

        let result = adf_coll
            .update_one(
                doc! { "name": problem_name, "username": username },
                doc! { "$set": { "adf": &adf, "acs_per_strategy.parse_only": &ac_and_graph } },
                None,
            )
            .await;

        if let Err(err) = result {
            log::error!("{err}");
        }
    }));

    HttpResponse::Ok().body("Parsing started...")
}

#[derive(Deserialize)]
struct SolveAdfProblemBody {
    strategy: Strategy,
}

#[put("/{problem_name}/solve")]
async fn solve_adf_problem(
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
    path: web::Path<String>,
    req_body: web::Json<SolveAdfProblemBody>,
) -> impl Responder {
    let problem_name = path.into_inner();
    let adf_problem_input: SolveAdfProblemBody = req_body.into_inner();
    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);

    let username = match identity.map(|id| id.id()) {
        None => {
            return HttpResponse::Unauthorized().body("You need to login to add an ADF problem.")
        }
        Some(Err(err)) => return HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => username,
    };

    let adf_problem = match adf_coll
        .find_one(doc! { "name": &problem_name, "username": &username }, None)
        .await
    {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(format!("ADF problem with name {problem_name} not found."))
        }
        Ok(Some(prob)) => prob,
    };

    let simp_adf: SimplifiedAdf = match adf_problem.adf {
        SimplifiedAdfOpt::None => {
            return HttpResponse::BadRequest().body("The ADF problem has not been parsed yet.")
        }
        SimplifiedAdfOpt::Error(err) => return HttpResponse::BadRequest().body(format!(
            "The ADF problem could not be parsed. Update it and try parsing it again. Error: {err}"
        )),
        SimplifiedAdfOpt::Some(adf) => adf,
    };

    let has_been_solved = match adf_problem_input.strategy {
        Strategy::Complete => adf_problem.acs_per_strategy.complete.is_some(),
        Strategy::Ground => adf_problem.acs_per_strategy.ground.is_some(),
        Strategy::Stable => adf_problem.acs_per_strategy.stable.is_some(),
        Strategy::StableCountingA => adf_problem.acs_per_strategy.stable_counting_a.is_some(),
        Strategy::StableCountingB => adf_problem.acs_per_strategy.stable_counting_b.is_some(),
        Strategy::StableNogood => adf_problem.acs_per_strategy.stable_nogood.is_some(),
    };

    // NOTE: we could also return the result here instead of throwing an error but I think the canonical way should just be to call the get endpoint for the problem.
    if has_been_solved {
        return HttpResponse::Conflict()
            .body("The ADF problem has already been solved with this strategy. You can just get the solution from the problem data directly.");
    }

    let username_clone = username.clone();
    let problem_name_clone = problem_name.clone();

    let acs_and_graphs_fut = timeout(
        COMPUTE_TIME,
        spawn_blocking(move || {
            let running_info = RunningInfo {
                username: username_clone,
                adf_name: problem_name_clone,
                task: Task::Solve(adf_problem_input.strategy),
            };

            app_state
                .currently_running
                .lock()
                .unwrap()
                .insert(running_info.clone());

            #[cfg(feature = "mock_long_computations")]
            std::thread::sleep(Duration::from_secs(20));

            let mut adf: Adf = Adf::from_ord_nodes_and_ac(
                simp_adf.ordering.into(),
                simp_adf.bdd.into_iter().map(Into::into).collect(),
                simp_adf
                    .ac
                    .into_iter()
                    .map(|t| Term(t.parse().unwrap()))
                    .collect(),
            );

            let acs: Vec<Ac> = match adf_problem_input.strategy {
                Strategy::Complete => adf.complete().collect(),
                Strategy::Ground => vec![adf.grounded()],
                Strategy::Stable => adf.stable().collect(),
                // TODO: INPUT VALIDATION: only allow this for hybrid parsing
                Strategy::StableCountingA => adf.stable_count_optimisation_heu_a().collect(),
                // TODO: INPUT VALIDATION: only allow this for hybrid parsing
                Strategy::StableCountingB => adf.stable_count_optimisation_heu_b().collect(),
                // TODO: support more than just default heuristics
                Strategy::StableNogood => adf
                    .stable_nogood(adf_bdd::adf::heuristics::Heuristic::default())
                    .collect(),
            };

            let acs_and_graphs: Vec<AcAndGraph> = acs
                .iter()
                .map(|ac| AcAndGraph {
                    ac: ac.iter().map(|t| t.0.to_string()).collect(),
                    graph: adf.into_double_labeled_graph(Some(ac)),
                })
                .collect();

            app_state
                .currently_running
                .lock()
                .unwrap()
                .remove(&running_info);

            acs_and_graphs
        }),
    );

    spawn(acs_and_graphs_fut.then(move |acs_and_graphs_res| async move {
        let acs_and_graphs_enum: AcsAndGraphsOpt = match acs_and_graphs_res {
            Err(err) => AcsAndGraphsOpt::Error(err.to_string()),
            Ok(Err(err)) => AcsAndGraphsOpt::Error(err.to_string()),
            Ok(Ok(acs_and_graphs)) => AcsAndGraphsOpt::Some(acs_and_graphs),
        };

        let result = adf_coll.update_one(doc! { "name": problem_name, "username": username }, match adf_problem_input.strategy {
            Strategy::Complete => doc! { "$set": { "acs_per_strategy.complete": &acs_and_graphs_enum } },
            Strategy::Ground => doc! { "$set": { "acs_per_strategy.ground": &acs_and_graphs_enum } },
            Strategy::Stable => doc! { "$set": { "acs_per_strategy.stable": &acs_and_graphs_enum } },
            Strategy::StableCountingA => doc! { "$set": { "acs_per_strategy.stable_counting_a": &acs_and_graphs_enum } },
            Strategy::StableCountingB => doc! { "$set": { "acs_per_strategy.stable_counting_b": &acs_and_graphs_enum } },
            Strategy::StableNogood => doc! { "$set": { "acs_per_strategy.stable_nogood": &acs_and_graphs_enum } },
        }, None).await;

        if let Err(err) = result {
            log::error!("{err}");
        }
    }));

    HttpResponse::Ok().body("Solving started...")
}

#[get("/{problem_name}")]
async fn get_adf_problem(
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
    path: web::Path<String>,
) -> impl Responder {
    let problem_name = path.into_inner();
    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);

    let username = match identity.map(|id| id.id()) {
        None => {
            return HttpResponse::Unauthorized().body("You need to login to get an ADF problem.")
        }
        Some(Err(err)) => return HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => username,
    };

    let adf_problem = match adf_coll
        .find_one(doc! { "name": &problem_name, "username": &username }, None)
        .await
    {
        Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
        Ok(None) => {
            return HttpResponse::NotFound()
                .body(format!("ADF problem with name {problem_name} not found."))
        }
        Ok(Some(prob)) => prob,
    };

    HttpResponse::Ok().json(AdfProblemInfo::from_adf_prob_and_tasks(
        adf_problem,
        &app_state.currently_running.lock().unwrap(),
    ))
}
