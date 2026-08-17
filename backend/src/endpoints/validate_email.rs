use actix_web::{HttpResponse, get, web};
use serde_json::json;
use std::collections::HashMap;

#[get("/validate_email")]
pub async fn validate_email(email: web::Query<HashMap<String, String>>) -> HttpResponse {
    tracing::info!("Validating email");
    tracing::warn!("The raw data received: {email:#?}");

    email
        .get("email_input")
        .map(std::string::String::as_str)
        .map_or_else(
            || {
                tracing::error!("Unable to get the email from the form");
                HttpResponse::Ok().body("Unable to get the email from the form")
            },
            |email| {
                tracing::warn!("Checking the email text");
                if email.contains('@') && email.contains('.') {
                    tracing::warn!("Acceptable email format, well done.");
                    HttpResponse::Ok()
                        .json(json!({"valid": true, "message": "Email format is valid"}))
                } else {
                    tracing::error!("Invalid email format, thus a bad request");
                    HttpResponse::Ok()
                        .json(json!({"valid": false, "message": "Invalid email format!"}))
                }
            },
        )
}
