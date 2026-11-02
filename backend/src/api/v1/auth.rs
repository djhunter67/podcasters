use actix_web::{
    HttpRequest, HttpResponse, post,
    web::{self, Data},
};
use futures::TryStreamExt;
use models::{DataBases, mongo_conf::get_user, rate_limit};
use mongodb::bson::{doc, oid};
use redis::{AsyncTypedCommands, aio};
use serde::{Deserialize, Serialize};
use shared::security::passworder::PassWorder;
use std::fmt;
use tracing::instrument;
use users::{Login, Users};

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
    let password_2: &str = &user_to_register.0.password_2;

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
        .collection::<Users>("users");

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

    tracing::warn!("PW Encrypted");

    // Save the user_to_register to the database
    let result_oid = db
        .insert_one(Users::new(email.as_str(), encrypted_pw.to_string()))
        .await;

    match result_oid {
        Ok(_oid) => {
            tracing::info!("Database save successful");

            let cache_key = format!("podcasters:user:auth:{email}");

            tracing::info!("Saving the cache-key to the cache-layer: {cache_key}");

            // let auth_data = LoginChecker::new(email, encrypted_pw.get());

            if let Ok(json_data) =
                // serde_json::to_string(&oid.inserted_id.as_object_id().expect("Oid not generated"))
                // No-OP
                serde_json::to_string(&encrypted_pw.to_string())
            // encrypted_pw.to_string()
            {
                tracing::warn!("the json data to be saved: {cache_key}::{json_data}");

                if let Err(err) = redis_client
                    .as_ref()
                    .clone()
                    .set_ex(&cache_key, encrypted_pw.to_string(), 3600) // 1 hour
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
    user_to_login: web::Json<Login>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Logiging in a user_to_register: {user_to_login:#?}");

    let _rate_limit: bool = rate_limit::rate_limit_check(&req);

    let user_email: String = user_to_login.0.email.clone();

    // Check redis first
    tracing::info!("Checking the cache-layer for: {user_email}");
    let cache_key = format!("podcasters:user:auth:{user_email}");

    // Get the user's key from when the user registered
    let cached_data: String = match redis_client.as_ref().clone().get(cache_key).await {
        Ok(cached_data) => cached_data.unwrap_or_default(),
        Err(err) => {
            tracing::warn!("No registration keys detected: {err}");
            String::new()
        }
    };

    tracing::warn!("The cached information to check against: {cached_data:#?}");

    // User stored in the database
    let user_auth: users::Users = if cached_data.is_empty() {
        // This case is if there is no session key returned from the browser
        // TODO: Change this from an error to a warn
        tracing::error!("cache-miss");

        let filter = mongodb::bson::doc! {
            "email":  user_email.clone()
        };

        // Missed the cache layer but found in the database
        match get_user(filter, &mongo_client).await {
            Some(user) => {
                tracing::info!("User found");
                user
            }
            None => {
                tracing::error!("User not found");
                return HttpResponse::NotFound().json(web::Json("Username not found"));
            }
        }
    } else {
        tracing::warn!("cache-hit");

        tracing::warn!("The cache: {cached_data}");
        // let oid: String = serde_json::from_str(&cached_data.to_string()).expect("Unable to get");

        // let filter = mongodb::bson::doc! {
        //     "_id":  ObjectId::parse_str(oid).expect("no parse availabe")
        // };

        // match get_user(filter, &mongo_client).await {
        //     Some(user) => {
        //         tracing::info!("User found");
        //         user
        //     }
        //     None => {
        //         tracing::error!("User not found");
        //         return HttpResponse::NotFound().json(web::Json("Potential query issues"));
        //     }
        // }

        let mut new_user: users::Users = users::Users::from(&user_to_login.0);

        new_user.set_pw(PassWorder::from(&cached_data));

        new_user
    };

    let user_pw = PassWorder::new(&user_to_login.0.password)
        .encrypt()
        .salt()
        .pepper();

    if let Ok(authed) = user_auth.pw_verify(
        &user_pw.to_string(), // &user_to_login.0.password,
    ) && authed
    {
        tracing::warn!("PASSWORD VERIFIED! -> True");
        let oid: oid::ObjectId = serde_json::from_str(&cached_data).unwrap_or_default();
        return HttpResponse::Accepted().json(web::Json(oid));
    }

    // THIS RETURN VAL IS TEMPORARY
    tracing::error!("PASSWORD INCORRECT");

    HttpResponse::Unauthorized().json("User login Invalid")
}
