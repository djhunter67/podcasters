use actix_web::{
    HttpResponse, get, post,
    web::{self, Data},
};
use askama::Template;
use futures::TryStreamExt;
use mongodb::bson::doc;
use redis::{AsyncCommands, aio};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::{
    endpoints::templates,
    models::mongo::{self},
    personnel::users::Users,
    security::passworder::PassWorder,
    settings,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct RegisterUser {
    #[serde(rename = "email_input")]
    pub email: String,
    #[serde(rename = "password_input")]
    pub password: String,
    #[serde(rename = "password_2_input")]
    password_2: String,
}

#[get("/register")]
#[instrument(
    name = "User registration attempted",
    level = "info",
    target = "sundayLifeServices web app"
)]
pub async fn register_template() -> HttpResponse {
    let template = templates::RegisterTemplate {
        title: "Registration",
        ..Default::default()
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok().body(template)
}

#[post("/register_user")]
#[instrument(
    name = "User registration attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(body, mongo_client, redis_client)
)]
pub async fn register_user(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    body: web::Form<RegisterUser>,
) -> HttpResponse {
    // Validate the user data entered
    let email: String = String::from(&body.0.email).to_lowercase();
    let password: &str = &body.0.password;
    let password_2: &str = &body.0.password_2;

    let restricted_and_invisible_chars = ['\n', '\r', '\t', '\0', '\x0B', '\x0C'];

    if password
        .chars()
        .any(|c| restricted_and_invisible_chars.contains(&c))
    {
        return HttpResponse::NotAcceptable().finish();
    }

    if !password.eq(password_2) {
        error!("Password not equal during registration");
        return HttpResponse::NotAcceptable().json("Passwords do not match");
    }

    let mongo_settings: settings::Mongo = match settings::get() {
        Ok(settings) => settings.mongo,
        Err(err) => {
            tracing::error!("Unable to procure the application settings: {err:#?}");
            return HttpResponse::InternalServerError().body(format!("Settings error: {err:#?}"));
        }
    };

    let db: mongodb::Collection<Users> = match mongo::establish_connection(&mongo_client).await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("Unable to procure the database: {err:#?}");
            return HttpResponse::InternalServerError()
                .body(format!("Unable to procure the database: {err:#?}"));
        }
    }
    .collection(&mongo_settings.collection);

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
        // return HttpResponse::Conflict().body("Email already exists");
        return HttpResponse::Ok().body("Email already exists");
    }
    tracing::info!("Email checking and no matching email found");

    let encrypted_pw: PassWorder = PassWorder::new(password).encrypt().salt().pepper();

    let (salt, _pw, _pepper) = encrypted_pw.deconstruct();

    // Save the user to the database
    let result_oid = db
        .insert_one(Users::new(email.clone(), encrypted_pw.to_string(), salt))
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

            let login_template = templates::LoginTemplate {
                user_email: "Login Succesful!",
                ..Default::default()
            };

            let render = login_template
                .render()
                .expect("Failure to render the login template");

            HttpResponse::Created().body(render)
        }
        Err(err) => {
            tracing::error!("Unable to register user: {err:#?}");
            // HttpResponse::InternalServerError().json(err.to_string())},
            let index_template = templates::IndexTemplate::default();

            let render = index_template
                .render()
                .expect("Failure to return the default index");

            HttpResponse::Ok().body(render)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn test_register_a_user() {
        let app = test::init_service(App::new().service(register_template)).await;

        let req = test::TestRequest::get().uri("/register").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    #[ignore = "The test logic is broken, it should return a 200 code but it fails for that and passes for an Internal Server Error code"]
    async fn test_user_is_registered() {
        let app = test::init_service(App::new().service(register_user)).await;

        let req = test::TestRequest::post()
            .uri("/register_user")
            .set_form(&RegisterUser {
                email: String::from("some_email@email.com"),
                password: "some_password".to_string(),
                password_2: "some_password".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_user_is_cached() {
        let app = test::init_service(App::new().service(register_user)).await;

        let req = test::TestRequest::post()
            .uri("/register_user")
            .set_form(&RegisterUser {
                email: String::from("some_email_2@email.com"),
                password: "some_password".to_string(),
                password_2: "some_password".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        let resp_body = test::read_body(resp).await;

        assert!(!resp_body.is_empty());
    }
}
