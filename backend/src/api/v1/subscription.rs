use actix_web::{HttpRequest, HttpResponse, web};
use models::redis_conf::authenticated_user_id;
use mongodb::bson::oid::ObjectId;
use podcasting::episode::Podcast;
use serde::{Deserialize, Serialize};
use shared::settings::Settings;
use tracing::instrument;

#[derive(Debug)]
pub enum SubscriptionError {
    NoPodcast(String),
    NoUser(String),
}

trait SubscriptionRepository {
    async fn subscribe(
        &self,
        user_id: &ObjectId,
        podcast_id: &ObjectId,
    ) -> anyhow::Result<(), SubscriptionError>;

    async fn unsubscribe(
        &self,
        user_id: &ObjectId,
        podcast_id: &ObjectId,
    ) -> anyhow::Result<(), SubscriptionError>;

    async fn subscriptions(
        &self,
        user_id: &ObjectId,
    ) -> anyhow::Result<Vec<Podcast>, SubscriptionError>;
}

#[derive(Debug, Deserialize, Serialize)]
struct Subscriptions {
    user_id: String,
    podcast_id: String,
}

impl SubscriptionRepository for Subscriptions {
    async fn subscribe(
        &self,
        user_id: &ObjectId,
        podcast_id: &ObjectId,
    ) -> anyhow::Result<(), SubscriptionError> {
        todo!()
    }

    async fn unsubscribe(
        &self,
        user_id: &ObjectId,
        podcast_id: &ObjectId,
    ) -> anyhow::Result<(), SubscriptionError> {
        todo!()
    }

    async fn subscriptions(
        &self,
        user_id: &ObjectId,
    ) -> anyhow::Result<Vec<Podcast>, SubscriptionError> {
        todo!()
    }
}

#[actix_web::post("/subscribe")]
#[instrument(
    name = "Episode Getter",
    level = "info",
    target = "Podcasting",
    skip(mongo_client, redis_client, settings)
)]
pub async fn subscribe(
    mut subscribe: web::Json<Subscriptions>,
    mongo_client: web::Data<mongodb::Client>,
    redis_client: web::Data<redis::aio::ConnectionManager>,
    settings: web::Data<Settings>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!(
        "subscribing a user: {} to a podcast: {}",
        subscribe.user_id,
        subscribe.podcast_id
    );

    let session_cookie = if let Some(cookie) = req.cookie("session_id") {
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {:#?}", req.connection_info());

        return HttpResponse::NotFound().json(web::Json(format!(
            "Potential illicit connection from: {}",
            req.connection_info().peer_addr().expect("No IP found")
        )));
    };

    tracing::warn!("{session_cookie:#?}");

    // authenticate the user
    // let var_name = &mut redis_client.as_ref().clone();
    // let oid: ObjectId = match authenticated_user_id(&session_cookie, &mongo_client, var_name).await
    // {
    //     Ok(id) => id,
    //     Err(err) => {
    //         tracing::error!("Authentication Error: {err}");
    //         return HttpResponse::InternalServerError().json(web::Json(err.to_string()));
    //     }
    // };

    HttpResponse::Ok().json(web::Json(subscribe))
}
