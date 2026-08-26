use actix_web::{HttpResponse, get};
use askama::Template;
use tracing::instrument;

use crate::endpoints::templates::DiscoveryTemplate;

/// Returns the main landing page for user who are not logged in
#[instrument(name = "Discovery page", level = "debug", target = "Podcasting Index")]
#[get("/discover")]
pub async fn discover() -> HttpResponse {
    let template = DiscoveryTemplate {
        ..Default::default()
    };

    let render = template.render().expect("Fail to render");

    HttpResponse::Ok().body(render)
}
