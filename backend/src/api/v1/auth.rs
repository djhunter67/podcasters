use std::fmt;

use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};
use models::DataBases;
use mongodb::bson::doc;
use redis::{AsyncTypedCommands, aio};
use serde::{Deserialize, Serialize};
use shared::{security::passworder::PassWorder, settings};
use tracing::instrument;
use users::Users;

use crate::api::errors::RegistrationError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Registration {
    pub email: String,
    pub password: String,
    pub password_2: String,
}

#[instrument(
    name = "Register a User",
    level = "info",
    target = "Podcasting",
    skip(user, mongo_client, redis_client, settings)
)]
#[post("/register")]
pub async fn register_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    settings: Data<settings::Settings>,
    user: web::Json<Registration>,
) -> HttpResponse {
    // Validate the user data entered
    let email: String = String::from(&user.0.email).to_lowercase();
    let password: &str = &user.0.password;
    let password_2: &str = &user.0.password_2;

    let restricted_and_invisible_chars = ['\n', '\r', '\t', '\0', '\x0B', '\x0C'];

    if password
        .chars()
        .any(|c| restricted_and_invisible_chars.contains(&c))
    {
        return HttpResponse::NotAcceptable().finish();
    }

    if !password.eq(password_2) {
        tracing::error!("Password not equal during registration");
        return HttpResponse::NotAcceptable().json("Passwords do not match");
    }

    let mongo_settings: settings::Mongo = match settings::get() {
        Ok(settings) => settings.mongo,
        Err(err) => {
            tracing::error!("Unable to procure the application settings: {err:#?}");
            return HttpResponse::InternalServerError()
                .json(web::Json(RegistrationError::Database(err.to_string())));
        }
    };

    // let db: mongodb::Collection<Users> =
    //     match mongo_client::establish_connection(&mongo_client).await {
    //         Ok(db) => db,
    //         Err(err) => {
    //             tracing::error!("Unable to procure the database: {err:#?}");
    //             return HttpResponse::InternalServerError()
    //                 .username(format!("Unable to procure the database: {err:#?}"));
    //         }
    //     }
    //     .collection(&mongo_settings.collection);

    let db = mongo_client
        .database(&DataBases::PodCast.to_string())
        .collection::<Users>(&DataBases::PodCast.to_string());

    // Check if the user exists
    let mut existing_query = db
        .find(doc! {
        "email": &email
        })
        .limit(1)
        .await
        .expect("");

    let result: Option<Users> = existing_query
        .try_next()
        .await
        .expect("no registered data found");

    if let Some(_data) = result {
        tracing::error!("Email already exists");
        // return HttpResponse::Conflict().user("Email already exists");
        return HttpResponse::AlreadyReported().json(web::Json("Email already exists"));
    }
    tracing::info!("Email checking and no matching email found");

    let encrypted_pw: PassWorder = PassWorder::new(password).encrypt().salt().pepper();

    let (salt, _pw, _pepper) = encrypted_pw.deconstruct();

    // Save the user to the database
    let result_oid = db
        .insert_one(Users::new(email.clone(), encrypted_pw.to_string()))
        .await;

    match result_oid {
        Ok(oid) => {
            tracing::info!("Database save successful");

            let cache_key = format!("user:auth:{email}");

            tracing::info!("Saving the cache-key to the cache-layer: {cache_key}");

            // let auth_data = LoginChecker::new(email, encrypted_pw.get());

            if let Ok(json_data) =
                serde_json::to_string(&oid.inserted_id.as_object_id().expect("Oid not generated"))
            {
                tracing::warn!("the json data to be saved: {cache_key}{json_data}");
                match redis_client
                    .as_ref()
                    .clone()
                    .set(&cache_key, json_data)
                    .await
                {
                    // change to 3200 for production
                    Ok(()) => (),
                    Err(err) => tracing::error!("Error saving to the cache layer -> {err:#?}"),
                }
            }

            HttpResponse::Created().json("User Created!")
        }
        Err(err) => {
            tracing::error!("Unable to register user: {err:#?}");

            HttpResponse::InternalServerError().json(RegistrationError::Database(err.to_string()))
        }
    }
}

impl fmt::Display for Registration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}::{}", self.email, self.password, self.password_2)
    }
}
