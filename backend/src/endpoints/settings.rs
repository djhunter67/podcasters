use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{HttpRequest, HttpResponse, get, post, web::Data};
use askama::Template;
use mongodb::{
    bson::{doc, oid},
    options::{self, UpdateModifications},
};
use redis::{AsyncCommands, aio};
use tracing::instrument;

use crate::{
    endpoints::templates::{self, IndexTemplate},
    models::{mongo, redis_conf::authenticated_user_id},
    personnel::users,
    security::{passworder::PassWorder, validate},
};

#[derive(Debug, MultipartForm)]
pub struct UserSettingsChange {
    #[multipart(rename = "email_update")]
    pub user_email: Option<Text<String>>,
    #[multipart(rename = "new_password")]
    pub new_pw: Option<Text<String>>,
    #[multipart(rename = "new_password_2")]
    pub new_pw_2: Option<Text<String>>,
    #[multipart(rename = "profile_image")]
    pub image: Option<TempFile>,
}

#[allow(clippy::future_not_send)]
#[get("/settings")]
#[instrument(
    name = "User settings",
    level = "info",
    target = "Load the settings landing page",
    skip(req, mongo_client, redis_client)
)]
pub async fn settings_template(
    req: HttpRequest,
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
) -> HttpResponse {
    tracing::info!("Settings page loading");

    match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(_oid) => {
            let session_id = if let Some(cookie) = req.cookie("session_id") {
                cookie.value().to_string()
            } else {
                tracing::error!("User cookie not found: {req:#?}");
                return HttpResponse::Unauthorized().body(format!(
                    "No session found: {:#?}",
                    req.cookies().expect("No cookies found")
                ));
            };

            // let mut red_conn = match redis_conf::establish_connection(&redis_client) {
            //     Ok(conn) => conn,
            //     Err(err) => {
            //         tracing::error!("Unable to acquire the cache layer connection: {err:#?}");
            //         return HttpResponse::InternalServerError()
            //             .body(format!("Cache layer error: {err:#?}"));
            //     }
            // };

            tracing::info!("Creating the session key");
            let session_key = format!("session:{session_id}");

            tracing::info!("Searching for the session key: {session_key}");
            let user: Option<String> = match redis_client.as_ref().clone().get(session_key).await {
                Ok(result) => result,
                Err(err) => {
                    tracing::error!("Error accessing the cache layer: {err:#?}");
                    return HttpResponse::InternalServerError()
                        .body(format!("Unable to acquire the cache layer: {err:#?}"));
                }
            };

            // Authenticate this endpoint
            let template = templates::SettingsTemplate {
                title: "Settings",
                is_logged_in: true,
                user_email: &user.unwrap_or_default(),
                ..Default::default()
            };

            let template = template.render().expect("Login page render error");

            return HttpResponse::Ok().body(template);
        }
        Err(err) => {
            tracing::error!("User not authorized to change the settings: {err:#?}");
            return HttpResponse::Unauthorized().json("User cookie expired or user not authorized");
        }
    }
}

/// All input are optional
#[allow(clippy::future_not_send)]
#[post("/settings_change")]
#[instrument(
    name = "User settings change",
    level = "info",
    target = "Load and process the settings page",
    skip(mongo_client, redis_client, req, body)
)]
pub async fn settings_change(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<aio::ConnectionManager>,
    req: HttpRequest,
    MultipartForm(body): MultipartForm<UserSettingsChange>,
) -> HttpResponse {
    let user_oid: oid::ObjectId =
        match authenticated_user_id(&req, &mongo_client, &redis_client).await {
            Ok(oid) => oid,
            Err(err) => {
                tracing::error!("Unable to authorize the user: {err:#?}");
                return HttpResponse::Unauthorized().json("User not authorized");
            }
        };
    let pw_1 = body.new_pw.as_ref().map(|pw| pw.as_str());

    let pw_2 = body.new_pw_2.as_ref().map(|pw| pw.as_str());

    if let Some(img) = &body.image {
        tracing::warn!(
            size = img.size / 1024,
            file_name = ?img.file_name,
            content_type = ?img.content_type,
        );

        let img_bytes: f32 = img.size as f32 / (1024.0 * 1024.0);

        // limit the output to 2 decimal places
        tracing::warn!("Image bytes size: {:.2} MB", img_bytes);
    } else {
        tracing::info!("No image uploaded");
    }

    if !pw_1.eq(&pw_2) {
        tracing::error!("Passwords do no match");
        // return HttpResponse::BadRequest().json("Passwords do not match");
        return HttpResponse::Ok().json("Passwords do not match");
    }

    let pw_1 = pw_1.unwrap_or("");

    if pw_1.is_empty() {
        tracing::info!("No password change requested");
        return HttpResponse::Ok().json("No password change requested");
    }

    match update_user_pw(pw_1, &mongo_client, &user_oid).await {
        Ok(()) => tracing::info!("Password update succeeded"),
        Err(err) => {
            tracing::error!("Password update failed{err:#?}");

            return HttpResponse::BadRequest().json(format!("Password update failed: {err}"));
        }
    }

    if let Some(new_email) = body.user_email.as_ref().map(|email| email.as_str()) {
        tracing::info!("Validating the new email: {new_email}");

        // Validate the email is actually an email
        if validate::email(new_email) {
            tracing::info!("New email is valid!");

            let _ = update_user_email(new_email, &mongo_client, &user_oid).await;
        }
    }

    let index_template = IndexTemplate {
        user_email: user_oid.to_string(),
        is_logged_in: true,
        ..Default::default()
    };

    match index_template.render() {
        Ok(rend) => HttpResponse::Ok().body(rend),
        Err(err) => {
            tracing::error!("Unable to render the index template from settings update");
            HttpResponse::InternalServerError().body(format!(
                "Unable to render the index page defaults: {err:#?}"
            ))
        }
    }
}

#[instrument(
    name = "User password change",
    level = "info",
    target = "Changing user password",
    skip(pw, mongo_client, user_oid)
)]
async fn update_user_pw(
    pw: &str,
    mongo_client: &Data<mongodb::Client>,
    user_oid: &oid::ObjectId,
) -> anyhow::Result<()> {
    let mongo_conn = mongo::establish_connection(mongo_client).await?;

    let passwdr: PassWorder = PassWorder::new(pw).encrypt().salt().pepper();
    let (salt, _encrypted, _pep) = passwdr.deconstruct();

    let filter = doc! {
    "_id": user_oid,
    };

    // The exact data to be updated
    let update_doc = doc! {
    "$set": doc! {
        "password_hash": passwdr.to_string(),
        "password_salt": salt,
    }
    };

    let options = options::FindOneAndUpdateOptions::builder()
        //     .sort(doc! { "date": -1 }) // Sort by date in descending order
        .return_document(options::ReturnDocument::After) // Return the updated document
        .build();

    let _user: users::Users = if let Some(user) = mongo_conn
        .collection::<users::Users>("Users")
        .find_one_and_update(filter, update_doc)
        .with_options(options)
        .await?
    {
        tracing::warn!("The user password was updated: {user:#?}");
        // user
        return Ok(());
    } else {
        tracing::error!("No user found when attempting to update the password");
        return Err(anyhow::Error::msg(
            "Unable to procure the user to update the password",
        ));
    };

    // let hash_pw: String = PassWorder::new(pw).encrypt().salt().pepper().to_string();

    // tracing::info!("Upadating the user password");
    // Save the users::Users struct to the database to commit
    // user.set_pw(pw);

    // tracing::warn!("The new hashed pw: {}", user.get_pw());

    // Ok(())
}

#[instrument(
    name = "User email change",
    level = "info",
    target = "Changing user email",
    skip(new_email, mongo_client, user_oid)
)]
async fn update_user_email(
    new_email: &str,
    mongo_client: &Data<mongodb::Client>,
    user_oid: &oid::ObjectId,
) -> anyhow::Result<()> {
    let mongo_conn = mongo::establish_connection(mongo_client).await?;

    let filter = doc! {
    "_id": user_oid,
    };

    // The exact data to be updated
    let update_doc = doc! {
    "$set": doc! {
        "email": new_email,
    }
    };

    let options = options::FindOneAndUpdateOptions::builder()
        //     .sort(doc! { "date": -1 }) // Sort by date in descending order
        .return_document(options::ReturnDocument::After) // Return the updated document
        .build();

    let _user: users::Users = if let Some(user) = mongo_conn
        .collection::<users::Users>("Users")
        .find_one_and_update(filter, update_doc)
        .with_options(options)
        .await?
    {
        tracing::warn!("Updated user data: {user:#?}");
        // user
        return Ok(());
    } else {
        tracing::error!("No user found when attempting to update the password");
        return Err(anyhow::Error::msg(
            "Unable to procure the user to update the password",
        ));
    };

    // user.set_email(new_email);

    // mongo_conn
    //     .collection::<users::Users>("Users")
    //     .update_one(filter, user)
    //     .await?;

    // tracing::warn!("The new user email: {}", user.get_email());

    // Ok(())
}

impl From<users::Users> for UpdateModifications {
    fn from(val: users::Users) -> Self {
        Self::Document(doc! {
            "$set": doc! {
        "email": val.get_email()
        }
        })
    }
}
