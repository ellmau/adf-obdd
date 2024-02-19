use std::collections::HashSet;
use std::sync::Mutex;

use actix_files as fs;
use actix_identity::IdentityMiddleware;
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::dev::{fn_service, ServiceRequest, ServiceResponse};
use actix_web::{web, App, HttpServer};
use fs::NamedFile;
use mongodb::Client;

#[cfg(feature = "cors_for_local_development")]
use actix_cors::Cors;

mod adf;
mod config;
mod double_labeled_graph;
mod user;
mod pmc_vis;

use adf::{
    add_adf_problem, delete_adf_problem, get_adf_problem, get_adf_problems_for_user,
    solve_adf_problem,
};
use config::{AppState, ASSET_DIRECTORY, COOKIE_DURATION};
use user::{
    create_username_index, delete_account, login, logout, register, update_user, user_info,
};
use pmc_vis::{
    pmc_vis_get_initial, pmc_vis_get_outgoing,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    // setup mongodb
    let mongodb_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(mongodb_uri)
        .await
        .expect("failed to connect to mongodb");
    create_username_index(&client).await;

    // cookie secret ket
    let secret_key = Key::generate();

    // needs to be set outside of httpserver closure to only create it once!
    let app_data = web::Data::new(AppState {
        mongodb_client: client.clone(),
        currently_running: Mutex::new(HashSet::new()),
    });

    HttpServer::new(move || {
        let app = App::new();

        #[cfg(feature = "cors_for_local_development")]
        let cors = Cors::default()
            .allowed_origin("http://localhost:1234")
            .allowed_origin("http://localhost:3000")
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);

        #[cfg(feature = "cors_for_local_development")]
        let app = app.wrap(cors);

        #[cfg(feature = "cors_for_local_development")]
        let cookie_secure = false;
        #[cfg(not(feature = "cors_for_local_development"))]
        let cookie_secure = true;

        app.app_data(app_data.clone())
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("adf-obdd-service-auth".to_owned())
                    .cookie_secure(cookie_secure)
                    .session_lifecycle(PersistentSession::default().session_ttl(COOKIE_DURATION))
                    .build(),
            )
            .service(
                web::scope("/users")
                    .service(register)
                    .service(delete_account)
                    .service(login)
                    .service(logout)
                    .service(user_info)
                    .service(update_user),
            )
            .service(
                web::scope("/adf")
                    .service(add_adf_problem)
                    .service(solve_adf_problem)
                    .service(get_adf_problem)
                    .service(delete_adf_problem)
                    .service(get_adf_problems_for_user),
            )
            .service(
                web::scope("/pmc-vis")
                    .service(pmc_vis_get_initial)
                    .service(pmc_vis_get_outgoing),
            )
            // this must be last to not override anything
            .service(
                fs::Files::new("/", ASSET_DIRECTORY)
                    .index_file("index.html")
                    .default_handler(fn_service(|req: ServiceRequest| async {
                        let (req, _) = req.into_parts();
                        let file =
                            NamedFile::open_async(format!("{ASSET_DIRECTORY}/index.html")).await?;
                        let res = file.into_response(&req);
                        Ok(ServiceResponse::new(req, res))
                    })),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
