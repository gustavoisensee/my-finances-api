use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::state::AppState;

pub type AppResult<T> = Result<T, AppError>;

const UNAUTHENTICATED_MESSAGE: &str =
    "Authentication required. Please provide a valid Firebase ID token.";

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{message}")]
    Rejection { status: StatusCode, message: String },
    #[error("unauthenticated")]
    Unauthenticated,
    #[error("admin access required")]
    AdminRequired,
    #[error("authorization error")]
    Authorization,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("{0}")]
    Internal(String),
}

impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }
}

/// Attached to 500 responses so [`internal_error_layer`] can log the cause and,
/// outside production, expose it in the body like the Express error handler did.
#[derive(Clone, Debug)]
pub struct InternalErrorDetail(pub String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::Validation(message) => message_response(StatusCode::BAD_REQUEST, &message),
            Self::NotFound(message) => message_response(StatusCode::NOT_FOUND, &message),
            Self::Rejection { status, message } => message_response(status, &message),
            Self::Unauthenticated => auth_response(
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                UNAUTHENTICATED_MESSAGE,
                "UNAUTHENTICATED",
            ),
            Self::AdminRequired => auth_response(
                StatusCode::FORBIDDEN,
                "Forbidden",
                "Admin access required.",
                "ADMIN_ACCESS_REQUIRED",
            ),
            Self::Authorization => auth_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                "An error occurred while verifying admin access.",
                "AUTHORIZATION_ERROR",
            ),
            Self::Database(err) => internal_response(err.to_string()),
            Self::Internal(detail) => internal_response(detail),
        }
    }
}

fn message_response(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "message": message }))).into_response()
}

fn auth_response(status: StatusCode, error: &str, message: &str, code: &str) -> Response {
    (
        status,
        Json(json!({ "error": error, "message": message, "code": code })),
    )
        .into_response()
}

fn internal_response(detail: String) -> Response {
    let mut response = message_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error");
    response
        .extensions_mut()
        .insert(InternalErrorDetail(detail));
    response
}

pub async fn internal_error_layer(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let response = next.run(request).await;

    let Some(InternalErrorDetail(detail)) =
        response.extensions().get::<InternalErrorDetail>().cloned()
    else {
        return response;
    };

    tracing::error!(%method, %path, error = %detail, "unexpected error");
    if state.config.is_production {
        response
    } else {
        message_response(StatusCode::INTERNAL_SERVER_ERROR, &detail)
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        let status = match rejection.status() {
            StatusCode::UNPROCESSABLE_ENTITY => StatusCode::BAD_REQUEST,
            other => other,
        };
        let message = match status {
            StatusCode::PAYLOAD_TOO_LARGE => "Request body is too large.".to_owned(),
            _ => rejection.body_text(),
        };
        Self::Rejection { status, message }
    }
}

impl From<PathRejection> for AppError {
    fn from(_: PathRejection) -> Self {
        Self::validation("Invalid id.")
    }
}

impl From<QueryRejection> for AppError {
    fn from(rejection: QueryRejection) -> Self {
        Self::validation(rejection.body_text())
    }
}

pub fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23505"))
}

pub fn is_foreign_key_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some("23503"))
}
