use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Data},
};
use askama::Template;
use mongodb::bson::oid;
use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    endpoints::templates::{self},
    models::mongo,
    personnel::users,
    security::session::create_session,
    settings,
};

/// # TODO:
///
/// Activity timestamps
/// Usage analytics
/// Retain login history
/// Delete user account
/// Successful logins
/// Failed login attempts
/// Password changed before
/// Data exported
#[derive(Deserialize, Debug, Serialize)]
pub struct LoginUser {
    #[serde(rename = "email_input")]
    pub email: String,
    #[serde(rename = "password_input")]
    pub password: String,
}

#[get("/login")]
#[instrument(
    name = "User login attempted",
    level = "info",
    target = "sundayLifeServices web app"
)]
pub async fn login_template() -> HttpResponse {
    debug!("Login page loaded");

    let template = templates::LoginTemplate::default();

    let template = template.render().expect("Login page render error");

    // testing the loading spinner
    // std::thread::sleep(std::time::Duration::from_secs(5));

    HttpResponse::Ok().body(template)
}

#[post("/login_user")]
#[instrument(
    name = "User login attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(body, mongo_client, redis_client)
)]
pub async fn login_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    body: web::Form<LoginUser>,
) -> impl Responder {
    debug!("The user data entered: {:#?}", body.0);

    // Validate the user data entered
    let user_email: &str = &body.0.email.clone();
    // let password: &str = body.0.password.as_str();

    let filter = mongodb::bson::doc! {
    "email":  user_email
    };

    // Check redis first
    tracing::info!("Checking the cache-layer for: {user_email}");
    let cache_key = format!("user:auth:{user_email}");
    // let mut redis_conn: r2d2::PooledConnection<redis::Client> =
    //     match redis_conf::establish_connection(&redis_client) {
    //         Ok(conn) => conn,
    //         Err(err) => {
    //             tracing::error!("Unable to procure the cache-layer connection: {err:#?}");
    //             return HttpResponse::InternalServerError()
    //                 .body(format!("Unable to procure the cache layer: {err:#?}"));
    //         }
    //     };

    // Get the user's key from when the user registered
    let cached_user: String = match redis_client.as_ref().clone().get(cache_key).await {
        Ok(cached_user) => cached_user,
        Err(err) => {
            tracing::warn!("No registration keys detected: {err}");
            String::new()
        }
    };

    tracing::warn!("The cached information to check against: {cached_user:#?}");

    let user_auth: users::Users = if cached_user.is_empty() {
        // This case is if there is no session key returned from the browser
        // TODO: Change this from an error to a warn
        tracing::error!("cache-miss");

        match cache_miss(filter, &mongo_client).await {
            Ok(user) => {
                tracing::info!("User, under cache-miss, is logged found");
                user
            }
            Err(err) => {
                tracing::error!("User not found: {err:#?}");
                users::Users::default()
            }
        }
    } else {
        tracing::warn!("cache-hit: {cached_user:#?}");

        let json_result: users::Users = users::Users::from(body);

        // json_result.set_pw(&json_result.get_pw());
        json_result
    };

    if let Ok(authed) = user_auth
        .pw_verify(&mongo_client, &mut redis_client.as_ref().clone(), None)
        .await
        && authed
    {
        tracing::warn!("PASSWORD VERIFIED! -> True");
        let oid: oid::ObjectId = serde_json::from_str(&cached_user).unwrap_or_default();
        return create_session(
            oid,
            &user_auth,
            redis_client.as_ref().clone(),
            &mongo_client,
        )
        .await;
    }

    // THIS RETURN VAL IS TEMPORARY
    tracing::error!("PASSWORD INCORRECT");
    // let default_template = IndexTemplate {
    //     user_email: format!("Invalid user entered credentials: {user_email}"),
    //     ..Default::default()
    // };

    // let render = default_template
    //     .render()
    //     .expect("The default render failed");

    HttpResponse::Unauthorized().body("User login Invalid")
}

async fn cache_miss(
    filter: mongodb::bson::Document,
    mongo_client: &Data<mongodb::Client>,
) -> anyhow::Result<users::Users> {
    let db: mongodb::Collection<users::Users> =
        match mongo::establish_connection(mongo_client).await {
            Ok(db) => db,
            Err(err) => {
                tracing::error!("Unable to procure the database: {err:#?}");
                return Err(anyhow::Error::msg(format!(
                    "Unable to procure the database: {err:#?}"
                )));
            }
        }
        .collection(
            &match settings::get() {
                Ok(settings) => settings,
                Err(err) => {
                    tracing::error!("Unable to procure database settings: {err:#?}");
                    return Err(anyhow::Error::msg(format!(
                        "Unable to procure database settings: {err:#?}"
                    )));
                }
            }
            .mongo
            .collection,
        );

    tracing::info!("Checking the cache-missed info against the db: {filter}");

    match db.find_one(filter).await {
        Ok(user) => user.map_or_else(
            || {
                tracing::error!("No user data found in the database");
                Ok(users::Users::default())
            },
            Ok,
        ),
        Err(err) => {
            tracing::error!("No conversion possible from Document to LoginChecker: {err}");
            Ok(users::Users::default())
        }
    }
}
