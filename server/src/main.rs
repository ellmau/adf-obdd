use actix_files as fs;
use actix_web::{post, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[cfg(feature = "cors_for_local_development")]
use actix_cors::Cors;
#[cfg(feature = "cors_for_local_development")]
use actix_web::http;

use adf_bdd::adf::{Adf, DoubleLabeledGraph};
use adf_bdd::parser::AdfParser;

#[derive(Deserialize)]
enum Strategy {
    ParseOnly,
    Ground,
    FirstComplete,
    FirstStable,
}

#[derive(Deserialize)]
struct SolveReqBody {
    code: String,
    strategy: Strategy,
}

#[derive(Serialize)]
struct SolveResBody {
    graph: DoubleLabeledGraph,
}

#[post("/solve")]
async fn solve(req_body: web::Json<SolveReqBody>) -> impl Responder {
    let input = &req_body.code;
    let strategy = &req_body.strategy;

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

    let ac = match strategy {
        Strategy::ParseOnly => None,
        Strategy::Ground => Some(adf.grounded()),
        // TODO: error handling if no such model exists!
        Strategy::FirstComplete => Some(adf.complete().next().unwrap()),
        // TODO: error handling if no such model exists!
        Strategy::FirstStable => Some(adf.stable().next().unwrap()),
    };

    let dto = adf.into_double_labeled_graph(ac.as_ref());

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

        App::new()
            .wrap(cors)
            .service(solve)
            // this mus be last to not override anything
            .service(fs::Files::new("/", "./assets").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    #[cfg(not(feature = "cors_for_local_development"))]
    let server = HttpServer::new(|| {
        App::new()
            .service(solve)
            // this mus be last to not override anything
            .service(fs::Files::new("/", "./assets").index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    server
}
