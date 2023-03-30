use std::time::Duration;

use actix_files as fs;
use actix_identity::{Identity, IdentityMiddleware};
use actix_session::config::PersistentSession;
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::rt::task::spawn_blocking;
use actix_web::rt::time::timeout;
use actix_web::{
    delete, post, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer, Responder,
    ResponseError,
};
use adf_bdd::datatypes::Term;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use mongodb::results::DeleteResult;
use mongodb::{bson::doc, options::IndexOptions, Client, IndexModel};
use serde::{Deserialize, Serialize};

use derive_more::{Display, Error};

#[cfg(feature = "cors_for_local_development")]
use actix_cors::Cors;

use adf_bdd::adf::{Adf, DoubleLabeledGraph};
use adf_bdd::adfbiodivine::Adf as BdAdf;
use adf_bdd::parser::AdfParser;

const THIRTY_MINUTES: actix_web::cookie::time::Duration =
    actix_web::cookie::time::Duration::minutes(30);

const ASSET_DIRECTORY: &str = "./assets";

const DB_NAME: &str = "adf-obdd";
const USER_COLL: &str = "users";
const ADF_COLL: &str = "adf-problems";

#[derive(Deserialize, Serialize)]
struct User {
    username: String,
    password: String,
}

// Creates an index on the "username" field to force the values to be unique.
async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(USER_COLL)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

// Add new user
#[post("/register")]
async fn register(client: web::Data<Client>, user: web::Json<User>) -> impl Responder {
    let mut user: User = user.into_inner();
    let user_coll = client.database(DB_NAME).collection(USER_COLL);

    let user_exists: bool = user_coll
        .find_one(doc! { "username": &user.username }, None)
        .await
        .ok()
        .flatten()
        .is_some();

    if user_exists {
        return HttpResponse::Conflict()
            .body("Username is already taken. Please pick another one!");
    }

    let pw = &user.password;
    let salt = SaltString::generate(&mut OsRng);
    let hashed_pw = Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .expect("Error while hashing password!")
        .to_string();

    user.password = hashed_pw;

    let result = user_coll.insert_one(user, None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("Registration successful!"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// Remove user
#[delete("/delete")]
async fn delete_account(client: web::Data<Client>, identity: Option<Identity>) -> impl Responder {
    let user_coll: mongodb::Collection<User> = client.database(DB_NAME).collection(USER_COLL);

    match identity.map(|id| id.id()) {
        None => HttpResponse::Unauthorized().body("You need to login to delete your account."),
        Some(Err(err)) => HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => {
            match user_coll
                .delete_one(doc! { "username": username }, None)
                .await
            {
                Ok(DeleteResult {
                    deleted_count: 0, ..
                }) => HttpResponse::InternalServerError().body("Account could not be deleted."),
                Ok(DeleteResult {
                    deleted_count: 1, ..
                }) => HttpResponse::Ok().body("Account deleted."),
                Ok(_) => unreachable!(
                    "delete_one removes at most one entry so all cases are covered already"
                ),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        }
    }
}

// Login
#[post("/login")]
async fn login(
    req: HttpRequest,
    client: web::Data<Client>,
    user_data: web::Json<User>,
) -> impl Responder {
    let username = &user_data.username;
    let pw = &user_data.password;
    let user_coll: mongodb::Collection<User> = client.database(DB_NAME).collection(USER_COLL);
    match user_coll
        .find_one(doc! { "username": username }, None)
        .await
    {
        Ok(Some(user)) => {
            let stored_hash = PasswordHash::new(&user.password).unwrap();
            let pw_valid = Argon2::default()
                .verify_password(pw.as_bytes(), &stored_hash)
                .is_ok();

            if pw_valid {
                Identity::login(&req.extensions(), username.to_string()).unwrap();
                HttpResponse::Ok().body("Login successful!")
            } else {
                HttpResponse::BadRequest().body("Invalid email or password")
            }
        }
        Ok(None) => HttpResponse::NotFound().body(format!(
            "No user found with username {}",
            &user_data.username
        )),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[delete("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.logout();

    HttpResponse::Ok().body("Logout successful!")
}

type Ac = Option<Vec<Term>>;

#[derive(Deserialize, Serialize)]
enum Parsing {
    Naive,
    Hybrid,
}

#[derive(Deserialize, Serialize)]
enum Strategy {
    ParseOnly,
    Ground,
    Complete,
    Stable,
    StableCountingA,
    StableCountingB,
    StableNogood,
}

#[derive(Serialize)]
struct AcsPerStrategy {
    parse_only: Ac,
    ground: Ac,
    complete: Ac,
    stable: Ac,
    stable_counting_a: Ac,
    stable_counting_b: Ac,
    stable_nogood: Ac,
}

#[derive(Serialize)]
struct AdfProblem {
    code: String,
    parsing_used: Parsing,
    adf: Adf,
    acs_per_strategy: AcsPerStrategy,
}

// #[get("/")]
// fn index() -> impl Responder {

// }

#[derive(Deserialize)]
struct SolveReqBody {
    code: String,
    parsing: Parsing,
    strategy: Strategy,
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

    let acs: Vec<Ac> = match strategy {
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

    // setup mongodb
    let mongodb_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".into());
    let client = Client::with_uri_str(mongodb_uri)
        .await
        .expect("failed to connect to mongodb");
    create_username_index(&client).await;

    // cookie secret ket
    let secret_key = Key::generate();

    let server = HttpServer::new(move || {
        let app = App::new();

        #[cfg(feature = "cors_for_local_development")]
        let cors = Cors::default()
            .allowed_origin("http://localhost:1234")
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        #[cfg(feature = "cors_for_local_development")]
        let app = app.wrap(cors);

        #[cfg(feature = "cors_for_local_development")]
        let cookie_secure = false;
        #[cfg(not(feature = "cors_for_local_development"))]
        let cookie_secure = true;

        app.app_data(web::Data::new(client.clone()))
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), secret_key.clone())
                    .cookie_name("adf-obdd-service-auth".to_owned())
                    .cookie_secure(cookie_secure)
                    .session_lifecycle(PersistentSession::default().session_ttl(THIRTY_MINUTES))
                    .build(),
            )
            .service(
                web::scope("/users")
                    .service(register)
                    .service(delete_account)
                    .service(login)
                    .service(logout),
            )
            .service(solve_with_timeout)
            // this mus be last to not override anything
            .service(fs::Files::new("/", ASSET_DIRECTORY).index_file("index.html"))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    server
}
