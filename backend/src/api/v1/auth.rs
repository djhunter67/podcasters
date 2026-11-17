use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};
use futures::TryStreamExt;
use models::{DataBases, mongo_conf::get_user};
use mongodb::bson::{doc, oid};
use redis::{AsyncTypedCommands, aio};
use serde::{Deserialize, Serialize};
use shared::security::passworder::PassWorder;
use std::{fmt, str::FromStr};
use tracing::instrument;
use users::{
    Auth, Users,
    auth::{Authenticated, Token, TokenGenerator, TokenType},
};

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
    skip(user_to_register, mongo_client, redis_client)
)]
#[post("/register")]
pub async fn register_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    user_to_register: web::Json<Registration>,
) -> HttpResponse {
    tracing::info!("Attempt to register a user_to_register");

    // let _rate_limit: bool = rate_limit::rate_limit_check(&req);

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
    skip(user_to_login, mongo_client, redis_client)
)]
#[post("/login")]
pub async fn login_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    user_to_login: web::Json<Auth>,
) -> HttpResponse {
    tracing::info!("Logiging in a user_to_register: {user_to_login:#?}");

    // let _rate_limit: bool = rate_limit::rate_limit_check(&req);

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
                return HttpResponse::NotFound().json(web::Json("Invalid login"));
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

    let user_pw = PassWorder::new(&user_to_login.0.password.expect("No pw available"))
        .encrypt()
        .salt()
        .pepper();

    if let Ok(authed) = user_auth.pw_verify(&user_pw.to_string())
        && authed
    {
        tracing::warn!("PASSWORD VERIFIED! -> True");

        let mut access_token: Token = Token::default();
        for token in TokenType::iter() {
            tracing::warn!("Token: {token}");
            let genned_token: Token = match Authenticated::generate_token(token) {
                Ok(val) => val,
                Err(err) => {
                    tracing::error!("The token generation has failed: {err}");
                    return HttpResponse::InternalServerError().json("{err}");
                }
            };

            if token.eq(&TokenType::Access) {
                access_token = genned_token.clone();
            }

            if let Err(err) = redis_client
                .as_ref()
                .clone()
                .set_ex(
                    format!("podcasters:auth:{token}:{}", user_to_login.0.email,),
                    genned_token.to_string(),
                    3600,
                ) // 1 hour
                .await
            {
                tracing::error!("Error saving to the cache layer -> {err:#?}")
            }
        }

        let return_hash_map: std::collections::HashMap<&str, String> =
            std::collections::HashMap::from([
                ("state", user_auth.get_status()),
                ("id", user_auth.get_oid()),
                ("token", access_token.get()),
            ]);

        return HttpResponse::Accepted().json(web::Json(return_hash_map));
    }

    // THIS RETURN VAL IS TEMPORARY
    tracing::error!("PASSWORD INCORRECT");

    HttpResponse::Unauthorized().json("User login Invalid")
}

#[instrument(
    name = "Logout a User",
    level = "info",
    target = "Podcasting",
    skip(user_to_logout, redis_client)
)]
#[post("/logout")]
pub async fn logout_user(
    user_to_logout: web::Json<Auth>,
    redis_client: Data<aio::ConnectionManager>,
) -> HttpResponse {
    tracing::info!("The user to logout: {user_to_logout:#?}");
    let oid: oid::ObjectId = oid::ObjectId::from_str(
        user_to_logout
            .oid
            .clone()
            .as_ref()
            .expect("Val is not present"),
    )
    .expect("Fail to get the OID");
    tracing::info!("OID passed in {oid:#?}");

    let cache_key = format!(
        "podcasters:auth:{}:{}",
        TokenType::Access,
        user_to_logout.email
    );

    let remove_cache_key: bool = match redis_client.as_ref().clone().del(&cache_key).await {
        Ok(val) => {
            tracing::warn!("The data is found: {val:#?}");
            val.eq(&1)
        }
        Err(err) => {
            tracing::error!("no cache data found: {err}");
            false
        }
    };

    tracing::info!("The cache key to remove: {remove_cache_key:#?}");

    HttpResponse::Accepted().json(web::Json(remove_cache_key))
}

#[instrument(
    name = "Refresh the cache timeout",
    level = "info",
    target = "Podcasting",
    skip(user_to_refresh, redis_client)
)]
#[post("/refresh")]
pub async fn refresh_cache(
    user_to_refresh: web::Json<Auth>,
    redis_client: Data<aio::ConnectionManager>,
) -> HttpResponse {
    let cache_key = format!(
        "podcasters:auth:{}:{}",
        TokenType::Refresh,
        user_to_refresh.email
    );

    tracing::warn!("The cache key: {cache_key}");

    let refresh: String = match redis_client.as_ref().clone().get(&cache_key).await {
        Ok(val) => val.unwrap_or_default(),
        Err(err) => {
            tracing::error!("Unable to procure the cache layer: {err}");
            String::from("Token procurement error")
            // return HttpResponse::InternalServerError().json(web::Json("Token retrieval error"));
        }
    };

    tracing::warn!("the refresh token: {refresh}");
    let mut ret_map: std::collections::HashMap<&str, String> = std::collections::HashMap::new();

    if refresh.is_empty() {
        return HttpResponse::NoContent().json(web::Json("No refresh token found"));
    } else {
        // The refresh token is validated, thus refresh the access token
        let access_token: Token = match Authenticated::generate_token(TokenType::Access) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("Access token generation failed while refreshing: {err}");
                return HttpResponse::InternalServerError().json("{err}");
            }
        };

        if let Err(err) = redis_client
            .as_ref()
            .clone()
            .set_ex(
                format!(
                    "podcasters:auth:{}:{}",
                    TokenType::Access,
                    user_to_refresh.0.email,
                ),
                access_token.to_string(),
                3600,
            ) // 30 days
            .await
        {
            tracing::error!("Error saving to the cache layer -> {err:#?}")
        }

        let refresh_token: Token = match Authenticated::generate_token(TokenType::Refresh) {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("Refresh token generation failed while refrshing: {err}");
                return HttpResponse::InternalServerError().json("{err}");
            }
        };

        if let Err(err) = redis_client
            .as_ref()
            .clone()
            .set_ex(
                format!(
                    "podcasters:auth:{}:{}",
                    TokenType::Refresh,
                    user_to_refresh.0.email,
                ),
                refresh_token.to_string(),
                25_920_000,
            ) // 30 days
            .await
        {
            tracing::error!("Error saving to the cache layer -> {err:#?}")
        }

        ret_map.insert("access", access_token.to_string());
        ret_map.insert("refresh", refresh_token.to_string());
    }

    tracing::warn!("The tokens to return: {ret_map:#?}");

    HttpResponse::Accepted().json(web::Json(ret_map))
}
