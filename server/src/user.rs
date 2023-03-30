use actix_identity::Identity;
use actix_web::{delete, post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use mongodb::results::DeleteResult;
use mongodb::{bson::doc, options::IndexOptions, Client, IndexModel};
use serde::{Deserialize, Serialize};

use crate::adf::AdfProblem;
use crate::config::{AppState, ADF_COLL, DB_NAME, USER_COLL};

#[derive(Deserialize, Serialize)]
pub(crate) struct User {
    pub(crate) username: String,
    pub(crate) password: Option<String>, // NOTE: Password being None indicates a temporary user
}

#[derive(Deserialize, Serialize)]
struct UserPayload {
    username: String,
    password: String,
}

// Creates an index on the "username" field to force the values to be unique.
pub(crate) async fn create_username_index(client: &Client) {
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

pub(crate) async fn username_exists(user_coll: &mongodb::Collection<User>, username: &str) -> bool {
    user_coll
        .find_one(doc! { "username": username }, None)
        .await
        .ok()
        .flatten()
        .is_some()
}

// Add new user
#[post("/register")]
async fn register(app_state: web::Data<AppState>, user: web::Json<UserPayload>) -> impl Responder {
    let mut user: UserPayload = user.into_inner();
    let user_coll = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(USER_COLL);

    if username_exists(&user_coll, &user.username).await {
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

    let result = user_coll
        .insert_one(
            User {
                username: user.username,
                password: Some(user.password),
            },
            None,
        )
        .await;
    match result {
        Ok(_) => HttpResponse::Ok().body("Registration successful!"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// Remove user
#[delete("/delete")]
async fn delete_account(
    app_state: web::Data<AppState>,
    identity: Option<Identity>,
) -> impl Responder {
    let user_coll: mongodb::Collection<User> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(USER_COLL);
    let adf_coll: mongodb::Collection<AdfProblem> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(ADF_COLL);

    match identity.map(|id| id.id()) {
        None => HttpResponse::Unauthorized().body("You need to login to delete your account."),
        Some(Err(err)) => HttpResponse::InternalServerError().body(err.to_string()),
        Some(Ok(username)) => {
            // Delete all adfs created by user
            match adf_coll
                .delete_many(doc! { "username": &username }, None)
                .await
            {
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
                Ok(DeleteResult {
                    deleted_count: 0, ..
                }) => HttpResponse::InternalServerError().body("Account could not be deleted."),
                Ok(DeleteResult {
                    deleted_count: _, ..
                }) => {
                    // Delete actual user
                    match user_coll
                        .delete_one(doc! { "username": &username }, None)
                        .await
                    {
                        Ok(DeleteResult {
                            deleted_count: 0, ..
                        }) => HttpResponse::InternalServerError()
                            .body("Account could not be deleted."),
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
    }
}

// Login
#[post("/login")]
async fn login(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    user_data: web::Json<UserPayload>,
) -> impl Responder {
    let username = &user_data.username;
    let pw = &user_data.password;
    let user_coll: mongodb::Collection<User> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(USER_COLL);
    match user_coll
        .find_one(doc! { "username": username }, None)
        .await
    {
        Ok(Some(user)) => {
            let stored_password = match &user.password {
                None => return HttpResponse::BadRequest().body("Invalid email or password"), // NOTE: login as tremporary user is not allowed
                Some(password) => password,
            };

            let stored_hash = PasswordHash::new(stored_password).unwrap();
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
async fn logout(app_state: web::Data<AppState>, id: Option<Identity>) -> impl Responder {
    let user_coll: mongodb::Collection<User> = app_state
        .mongodb_client
        .database(DB_NAME)
        .collection(USER_COLL);

    match id {
        None => HttpResponse::Unauthorized().body("You are not logged in."),
        Some(id) => match id.id() {
            Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            Ok(username) => {
                let user: User = match user_coll
                    .find_one(doc! { "username": &username }, None)
                    .await
                {
                    Ok(Some(user)) => user,
                    Ok(None) => {
                        return HttpResponse::NotFound()
                            .body(format!("No user found with username {}", &username))
                    }
                    Err(err) => return HttpResponse::InternalServerError().body(err.to_string()),
                };

                if user.password.is_none() {
                    HttpResponse::BadRequest().body("You are logged in as a temporary user so we won't log you out because you will not be able to login again. If you want to be able to login again, set a password. Otherwise your session will expire automatically at a certain point.")
                } else {
                    id.logout();
                    HttpResponse::Ok().body("Logout successful!")
                }
            }
        },
    }
}
