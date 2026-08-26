use actix_web::{HttpResponse, get};
use askama::Template;
use tracing::instrument;

use super::templates::IndexTemplate;

/// Returns the main landing page for user who are not logged in
#[instrument(name = "Index page", level = "debug", target = "Podcasting Index")]
#[get("/")]
pub async fn index() -> HttpResponse {
    let template = IndexTemplate {
        ..Default::default()
    };

    let render = template.render().expect("Fail to render");

    HttpResponse::Ok().body(render)
}
