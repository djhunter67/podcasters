use actix_web::{HttpResponse, web};
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
    mut subscript: web::Json<Subscriptions>,
    mongo_client: web::Data<mongodb::Client>,
    redis_client: web::Data<redis::aio::ConnectionManager>,
    settings: web::Data<Settings>,
) -> HttpResponse {
    tracing::info!(
        "subscribing a user: {} to a podcast: {}",
        subscript.user_id,
        subscript.podcast_id
    );

    HttpResponse::Ok().json(web::Json(subscript))
}
