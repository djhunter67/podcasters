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
mod tests {
    use actix_web::{App, http::StatusCode, test};

    use crate::api::health;
    // Import your specific health handler and app configuration

    #[actix_web::test]
    async fn test_health_endpoint() {
        // 1. Initialize the test service with your App configuration
        let app = test::init_service(
            App::new()
                // If your health handler requires app data (e.g., DB pool), add it here
                // .app_data(web::Data::new(pool))
                .service(health),
        )
        .await;

        // 2. Create a test request for the /v1/health endpoint
        let req = test::TestRequest::get().uri("/v1/health").to_request();

        // 3. Call the service with the request
        let resp = test::call_service(&app, req).await;

        // 4. Assert the response status is OK (200)
        assert_eq!(resp.status(), StatusCode::OK);

        // Optional: Assert specific headers or body content
        // let body = test::read_body(resp).await;
        // assert_eq!(body, b"OK");
    }
}
