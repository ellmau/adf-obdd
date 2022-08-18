use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use adf_bdd::adf::Adf;
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
    node_labels: Vec<String>,
    // every node gets this label containing multiple entries (it might be empty)
    tree_root_labels: Vec<Vec<String>>,
    edges: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct SolveReqBody {
    adf_input: String,
}

#[derive(Serialize)]
struct SolveResBody {
    graph: DoubleLabeledGraph,
}

#[post("/solve")]
async fn solve(req_body: web::Json<SolveReqBody>) -> impl Responder {
    let input = &req_body.adf_input;

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

    "Hello World"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    HttpServer::new(|| App::new().service(root).service(solve))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
