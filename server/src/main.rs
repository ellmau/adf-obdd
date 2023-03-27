use std::time::Duration;

use actix_files as fs;
use actix_web::rt::task::spawn_blocking;
use actix_web::rt::time::timeout;
use actix_web::{post, web, App, HttpServer, Responder, ResponseError};
use serde::{Deserialize, Serialize};

use derive_more::{Display, Error};

#[cfg(feature = "cors_for_local_development")]
use actix_cors::Cors;

use adf_bdd::adf::{Adf, DoubleLabeledGraph};
use adf_bdd::adfbiodivine::Adf as BdAdf;
use adf_bdd::parser::AdfParser;

#[derive(Deserialize)]
enum Parsing {
    Naive,
    Hybrid,
}

#[derive(Deserialize)]
enum Strategy {
    ParseOnly,
    Ground,
    Complete,
    Stable,
    StableCountingA,
    StableCountingB,
    StableNogood,
}

#[derive(Deserialize)]
struct SolveReqBody {
    code: String,
    parsing: Parsing,
    strategy: Strategy,
}

#[derive(Serialize)]
struct SolveResBody {
    graph: DoubleLabeledGraph,
}

fn solve(req_body: web::Json<SolveReqBody>) -> impl Responder {
    let input = &req_body.code;
    let parsing = &req_body.parsing;
    let strategy = &req_body.strategy;

    let parser = AdfParser::default();
    match parser.parse()(input) {
        Ok(_) => log::info!("[Done] parsing"),
        Err(e) => {
            log::error!("Error during parsing:\n{} \n\n cannot continue, panic!", e);
            panic!("Parsing failed, see log for further details")
        }
    }

    let mut adf = match parsing {
        Parsing::Naive => Adf::from_parser(&parser),
        Parsing::Hybrid => {
            let bd_adf = BdAdf::from_parser(&parser);
            log::info!("[Start] translate into naive representation");
            let naive_adf = bd_adf.hybrid_step_opt(false);
            log::info!("[Done] translate into naive representation");

            naive_adf
        }
    };

    log::debug!("{:?}", adf);

    let acs = match strategy {
        Strategy::ParseOnly => vec![None],
        Strategy::Ground => vec![Some(adf.grounded())],
        Strategy::Complete => adf.complete().map(Some).collect(),
        Strategy::Stable => adf.stable().map(Some).collect(),
        // TODO: INPUT VALIDATION: only allow this for hybrid parsing
        Strategy::StableCountingA => adf.stable_count_optimisation_heu_a().map(Some).collect(),
        // TODO: INPUT VALIDATION: only allow this for hybrid parsing
        Strategy::StableCountingB => adf.stable_count_optimisation_heu_b().map(Some).collect(),
        // TODO: support more than just default heuristics
        Strategy::StableNogood => adf
            .stable_nogood(adf_bdd::adf::heuristics::Heuristic::default())
            .map(Some)
            .collect(),
    };

    let dto: Vec<DoubleLabeledGraph> = acs
        .iter()
        .map(|ac| adf.into_double_labeled_graph(ac.as_ref()))
        .collect();

    web::Json(dto)
}

#[derive(Debug, Display, Error)]
#[display(
    fmt = "Endpoint {} timed out. Probably your ADF problem is too complicated :(",
    endpoint
)]
struct Timeout {
    endpoint: &'static str,
}

impl ResponseError for Timeout {}

#[post("/solve")]
async fn solve_with_timeout(req_body: web::Json<SolveReqBody>) -> impl Responder {
    timeout(Duration::from_secs(20), spawn_blocking(|| solve(req_body)))
        .await
        .map(|ok| {
            ok.expect(
                "An error in the spawned solve thread occurred. Timeouts are handled separately.",
            )
        })
        .map_err(|_| Timeout { endpoint: "/solve" })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    #[cfg(feature = "cors_for_local_development")]
    let server = HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .service(solve_with_timeout)
            // this mus be last to not override anything
            .service(fs::Files::new("/", "./assets").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    #[cfg(not(feature = "cors_for_local_development"))]
    let server = HttpServer::new(|| {
        App::new()
            .service(solve_with_timeout)
            // this mus be last to not override anything
            .service(fs::Files::new("/", "./assets").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    server
}
