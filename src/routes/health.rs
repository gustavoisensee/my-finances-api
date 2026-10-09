use axum::Json;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use serde_json::json;

pub async fn index() -> Html<&'static str> {
    Html("Index")
}

pub async fn status() -> Html<&'static str> {
    Html("Ok")
}

pub async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": "Not Found" })),
    )
}
