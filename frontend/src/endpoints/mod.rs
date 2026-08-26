pub mod discover;
pub mod index;
pub mod podcasts;
pub mod templates;

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

#[cfg(test)]
mod test {
    use actix_web::{App, http::StatusCode, middleware, test, web};

    use crate::endpoints;

    #[actix_web::test]
    async fn health_endpoint_is_configured_correctly() {
        let app = test::init_service(
            App::new()
                .wrap(
                    middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))),
                )
                .service(web::scope("/v1").service(endpoints::health)),
        )
        .await;

        let request = test::TestRequest::get().uri("/v1/health").to_request();

        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;

        assert_eq!(body.as_ref(), b"healthy!");
    }
}
