use actix_web::{HttpResponse, get, http::header::ContentType};
use askama::Template;
use tracing::instrument;

use crate::endpoints::templates::ErrorPage;

#[get("/error")]
#[instrument(
    name = "Serving application errors",
    level = "info",
    target = "portfolio_site"
)]
pub async fn error() -> HttpResponse {
    tracing::info!("{}", format!("Serving error page"));

    let var_name = ErrorPage {
        title: "Error",
        code: 400,
        error: "Succint error message",
        message: "Contact the developer directly",
    };

    let rendered = var_name.render().expect("Failed to render template");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered)
}
