use actix_web::{
    HttpRequest, HttpResponse, post,
    web::{self, Data},
};
use futures::TryStreamExt;
use models::{DataBases, rate_limit};
use mongodb::bson::doc;
use redis::{AsyncTypedCommands, aio};
use serde::{Deserialize, Serialize};
use shared::security::passworder::PassWorder;
use std::fmt;
use tracing::instrument;
use users::Users;

use crate::api::errors::RegistrationError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Registration {
    pub email: String,
    pub password: String,
    pub password_2: Option<String>,
}

#[instrument(
    name = "Register a User",
    level = "info",
    target = "Podcasting",
    skip(user_to_register, mongo_client, redis_client, req)
)]
#[post("/register")]
pub async fn register_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    user_to_register: web::Json<Registration>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Attempt to register a user_to_register");

    let _rate_limit: bool = rate_limit::rate_limit_check(&req);

    // Validate the user_to_register data entered
    let email: String = String::from(&user_to_register.0.email).to_lowercase();
    let password: &str = &user_to_register.0.password;
    let password_2: &str = match &user_to_register.0.password_2 {
        Some(val) => val,
        None => {
            tracing::error!("Second password not entered.");
            ""
        }
    };

    let restricted_and_invisible_chars = ['\n', '\r', '\t', '\0', '\x0B', '\x0C'];

    if password
        .chars()
        .any(|c| restricted_and_invisible_chars.contains(&c))
        || password_2
            .chars()
            .any(|c| restricted_and_invisible_chars.contains(&c))
    {
        return HttpResponse::NotAcceptable().json("Password contains invalid characters");
    }

    if !password.eq(password_2) {
        tracing::error!("Password not equal during registration");
        return HttpResponse::NotAcceptable().json("Passwords do not match");
    }

    let db: mongodb::Collection<Users> = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Users>("user_to_registers");

    // Check if the user_to_register exists
    let mut existing_query: mongodb::Cursor<Users> = db
        .find(doc! {
        "email": &email
        })
        .limit(1)
        .await
        .expect("No user_to_register found");

    let result: Option<Users> = existing_query
        .try_next()
        .await
        .expect("no registered data found");

    if let Some(_data) = result {
        tracing::error!("Email already exists");
        // return HttpResponse::Conflict().user_to_register("Email already exists");
        return HttpResponse::AlreadyReported().json(web::Json("Email already exists"));
    }
    tracing::info!("Email checking and no matching email found");

    let encrypted_pw: PassWorder = PassWorder::new(password).encrypt().salt().pepper();

    tracing::warn!("Encrypted PW");

    // let (_salt, _pw, _pepper) = encrypted_pw.deconstruct();

    // Save the user_to_register to the database
    let result_oid = db
        .insert_one(Users::new(email.clone(), encrypted_pw.to_string()))
        .await;

    match result_oid {
        Ok(oid) => {
            tracing::info!("Database save successful");

            let cache_key = format!("podcasters:user_to_register:auth:{email}");

            tracing::info!("Saving the cache-key to the cache-layer: {cache_key}");

            // let auth_data = LoginChecker::new(email, encrypted_pw.get());

            if let Ok(json_data) =
                serde_json::to_string(&oid.inserted_id.as_object_id().expect("Oid not generated"))
            {
                tracing::warn!("the json data to be saved: {cache_key}{json_data}");
                if let Err(err) = redis_client
                    .as_ref()
                    .clone()
                    .set_ex(&cache_key, json_data, 3600)
                    .await
                {
                    tracing::error!("Error saving to the cache layer -> {err:#?}")
                }
            }

            HttpResponse::Created().json("User Created!")
        }
        Err(err) => {
            tracing::error!("Unable to register user_to_register: {err:#?}");

            HttpResponse::InternalServerError().json(RegistrationError::Database(err.to_string()))
        }
    }
}

impl fmt::Display for Registration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}::{:?}",
            self.email, self.password, self.password_2
        )
    }
}

#[instrument(
    name = "Login a User",
    level = "info",
    target = "Podcasting",
    skip(user_to_login, mongo_client, redis_client, req)
)]
#[post("/login")]
pub async fn login_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    user_to_login: web::Json<Registration>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Logiging in a user_to_register");

    tracing::warn!("Request: {req:#?}");

    // HttpResponse::Accepted().json(web::Json("User login attempted"))
    HttpResponse::NotImplemented().json(web::Json("User login attempted"))
}
