use actix_web::HttpRequest;
use tracing::instrument;

#[instrument(
    name = "Register a User",
    level = "info",
    target = "Podcasting",
    skip(req)
)]
pub fn rate_limit_check(req: &HttpRequest) -> bool {
    tracing::info!("Enforce rate limit checks");

    let ip = req.connection_info().peer_addr().map_or_else(
        || String::from("No connection info"),
        std::string::ToString::to_string,
    );

    tracing::info!("The IP to check: {:#?}", ip);

    false
}

#[cfg(test)]
mod tests {}
