use actix_web::{App, http::StatusCode, middleware, test, web};

use backend::api;

#[actix_web::test]
async fn health_endpoint_is_configured_correctly() {
    let app = test::init_service(
        App::new()
            .wrap(middleware::DefaultHeaders::new().add(("X-Version", env!("CARGO_PKG_VERSION"))))
            .service(web::scope("/v1").service(api::health)),
    )
    .await;

    let request = test::TestRequest::get().uri("/v1/health").to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let version = response
        .headers()
        .get("X-Version")
        .expect("X-Version header should be present");

    assert_eq!(version.to_str().unwrap(), env!("CARGO_PKG_VERSION"));

    let body = test::read_body(response).await;

    assert_eq!(body.as_ref(), b"healthy!");
}

#[actix_web::test]
async fn invalid_route_returns_not_found() {
    let app = test::init_service(App::new().service(web::scope("/v1").service(api::health))).await;

    let request = test::TestRequest::get()
        .uri("/v1/does-not-exist")
        .to_request();

    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
