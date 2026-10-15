use axum::http::{HeaderValue, Method};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::config::Config;

/// Equivalent of `cors({ origin: allowedOrigins })`: only allowlisted origins,
/// the `cors` package's default methods, and requested headers mirrored back.
pub fn layer(config: &Config) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .allowed_origins()
        .into_iter()
        .map(HeaderValue::from_static)
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::HEAD,
            Method::PUT,
            Method::PATCH,
            Method::POST,
            Method::DELETE,
        ])
        .allow_headers(AllowHeaders::mirror_request())
}
