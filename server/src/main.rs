use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[cfg(feature = "cors_for_local_development")]
use actix_cors::Cors;

use adf_bdd::adf::Adf;
use adf_bdd::datatypes::BddNode;
use adf_bdd::datatypes::Var;
use adf_bdd::parser::AdfParser;

#[get("/")]
async fn root() -> impl Responder {
    // TODO: this should serve the static files for the react frontend
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Serialize)]
// This is a DTO for the graph output
struct DoubleLabeledGraph {
    // number of nodes equals the number of node labels
    // nodes implicitly have their index as their ID
    node_labels: HashMap<usize, String>,
    // every node gets this label containing multiple entries (it might be empty)
    tree_root_labels: HashMap<usize, Vec<String>>,
    lo_edges: Vec<(usize, usize)>,
    hi_edges: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct SolveReqBody {
    code: String,
}

#[derive(Serialize)]
struct SolveResBody {
    graph: DoubleLabeledGraph,
}

#[post("/solve")]
async fn solve(req_body: web::Json<SolveReqBody>) -> impl Responder {
    let input = &req_body.code;

    let parser = AdfParser::default();
    match parser.parse()(input) {
        Ok(_) => log::info!("[Done] parsing"),
        Err(e) => {
            log::error!("Error during parsing:\n{} \n\n cannot continue, panic!", e);
            panic!("Parsing failed, see log for further details")
        }
    }
    log::info!("[Done] parsing");

    let mut adf = Adf::from_parser(&parser);

    log::debug!("{:?}", adf);

    // TODO: as first test: turn full graph with initial ac into DoubleLabeledGraph DTO and return it

    // get relevant nodes from bdd and ac
    let mut node_indices: HashSet<usize> = HashSet::new();
    let mut new_node_indices: HashSet<usize> = adf.ac.iter().map(|term| term.value()).collect();

    while !new_node_indices.is_empty() {
        node_indices = node_indices.union(&new_node_indices).map(|i| *i).collect();
        new_node_indices = HashSet::new();

        for node_index in &node_indices {
            let lo_node_index = adf.bdd.nodes[*node_index].lo().value();
            if !node_indices.contains(&lo_node_index) {
                new_node_indices.insert(lo_node_index);
            }

            let hi_node_index = adf.bdd.nodes[*node_index].hi().value();
            if !node_indices.contains(&hi_node_index) {
                new_node_indices.insert(hi_node_index);
            }
        }
    }

    let node_labels: HashMap<usize, String> =
        adf.bdd
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| node_indices.contains(i))
            .map(|(i, &node)| {
                let value_part = match node.var() {
                    Var::TOP => "TOP".to_string(),
                    Var::BOT => "BOT".to_string(),
                    _ => adf.ordering.name(node.var()).expect(
                        "name for each var should exist; special cases are handled separately",
                    ),
                };

                (i, value_part)
            })
            .collect();

    let tree_root_labels: HashMap<usize, Vec<String>> = adf.ac.iter().enumerate().fold(
        adf.bdd
            .nodes
            .iter()
            .enumerate()
            .filter(|(i, _)| node_indices.contains(i))
            .map(|(i, _)| (i, vec![]))
            .collect(),
        |mut acc, (root_for, root_node)| {
            acc.get_mut(&root_node.value())
                .expect("we know that the index will be in the map")
                .push(adf.ordering.name(Var(root_for)).expect(
                    "name for each var should exist; special cases are handled separately",
                ));

            acc
        },
    );

    let lo_edges: Vec<(usize, usize)> = adf
        .bdd
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| node_indices.contains(i))
        .filter(|(_, node)| !vec![Var::TOP, Var::BOT].contains(&node.var()))
        .map(|(i, &node)| (i, node.lo().value()))
        .collect();

    let hi_edges: Vec<(usize, usize)> = adf
        .bdd
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| node_indices.contains(i))
        .filter(|(_, node)| !vec![Var::TOP, Var::BOT].contains(&node.var()))
        .map(|(i, &node)| (i, node.hi().value()))
        .collect();

    log::debug!("{:?}", node_labels);
    log::debug!("{:?}", tree_root_labels);
    log::debug!("{:?}", lo_edges);
    log::debug!("{:?}", hi_edges);

    let dto = DoubleLabeledGraph {
        node_labels,
        tree_root_labels,
        lo_edges,
        hi_edges,
    };

    web::Json(dto)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    #[cfg(feature = "cors_for_local_development")]
    let server = HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:1234")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new().wrap(cors).service(root).service(solve)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await;

    #[cfg(not(feature = "cors_for_local_development"))]
    let server = HttpServer::new(|| App::new().service(root).service(solve))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await;

    server
}
