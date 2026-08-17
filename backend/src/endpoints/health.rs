use actix_web::{HttpResponse, Responder, get};
use tracing::{info, instrument};

#[get("/health_check")]
#[instrument(name = "Health check", target = "demo_web_app", level = "info")]
pub async fn health_check() -> impl Responder {
    info!("Health check endpoint called.");
    HttpResponse::Ok().finish()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use actix_web::{App, test};

    use super::super::health::health_check;

    #[actix_web::test]
    async fn test_get_health() {
        let app = test::init_service(App::new().service(health_check)).await;
        let req = test::TestRequest::get().uri("/health_check").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
