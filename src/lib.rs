pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod extract;
pub mod models;
pub mod routes;
pub mod security;
pub mod state;
pub mod telemetry;
pub mod time;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;

pub use state::AppState;

/// The whole API as a single router, shared by the Vercel function, the local
/// dev server and the integration tests.
pub fn build_app(state: AppState) -> Router {
    let router = routes::router()
        .fallback(routes::health::not_found)
        .method_not_allowed_fallback(routes::health::not_found)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            error::internal_error_layer,
        ))
        .layer(DefaultBodyLimit::max(security::BODY_LIMIT_BYTES))
        .layer(security::cors::layer(&state.config));

    security::headers::apply(router).with_state(state)
}
