use actix_web::{HttpResponse, get};
use askama::Template;
use tracing::instrument;

use crate::endpoints::templates::PodcastTemplate;

/// Returns the main landing page for user who are not logged in
#[instrument(name = "Podcasts page", level = "debug", target = "Podcasting Index")]
#[get("/podcasts")]
pub async fn podcasts() -> HttpResponse {
    let template = PodcastTemplate {
        ..Default::default()
    };

    let render = template.render().expect("Fail to render");

    HttpResponse::Ok().body(render)
}
