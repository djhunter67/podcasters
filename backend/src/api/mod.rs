#[actix_web::get("/health")]
#[tracing::instrument(
    name = "Health endpoint",
    level = "debug",
    target = "Podcaster API health"
)]
pub async fn health() -> actix_web::HttpResponse {
    tracing::info!("The API HEALTH endpoint");
    actix_web::HttpResponse::Ok().body("healthy!")
}
