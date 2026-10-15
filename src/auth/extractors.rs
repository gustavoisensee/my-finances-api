use std::collections::HashSet;

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use sqlx::PgPool;

use super::firebase::FirebaseClaims;
use crate::error::AppError;
use crate::state::AppState;
use crate::time;

/// A request carrying a valid Firebase ID token (`Authorization: Bearer <token>`).
pub struct VerifiedToken {
    pub uid: String,
    pub claims: FirebaseClaims,
}

impl FromRequestParts<AppState> for VerifiedToken {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .ok_or(AppError::Unauthenticated)?;

        let claims = state.verifier.verify(token).await.map_err(|err| {
            tracing::debug!(reason = %err, "rejected Firebase ID token");
            AppError::Unauthenticated
        })?;

        Ok(Self {
            uid: claims.sub.clone(),
            claims,
        })
    }
}

/// An authenticated user whose `User` row is created on first sign-in.
pub struct AuthUser {
    pub uid: String,
    pub user_id: i32,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = VerifiedToken::from_request_parts(parts, state).await?;
        let user_id = ensure_user(&state.db, &token.uid).await?;
        Ok(Self {
            uid: token.uid,
            user_id,
        })
    }
}

/// An authenticated administrator: custom claim `admin: true`, `role: "admin"`,
/// or a uid listed in `ADMIN_FIREBASE_UIDS`.
pub struct AdminUser {
    pub uid: String,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = VerifiedToken::from_request_parts(parts, state).await?;
        if is_admin(&token.claims, &token.uid, &state.config.admin_uids) {
            Ok(Self { uid: token.uid })
        } else {
            Err(AppError::AdminRequired)
        }
    }
}

pub fn is_admin(claims: &FirebaseClaims, uid: &str, allowlist: &HashSet<String>) -> bool {
    claims.has_admin_claim() || allowlist.contains(uid)
}

/// Returns the user's id, inserting the row on first sign-in. The hot path is a
/// read; the insert only happens once per user.
pub async fn ensure_user(db: &PgPool, firebase_uid: &str) -> Result<i32, AppError> {
    let id = sqlx::query_scalar!(
        r#"
        WITH existing AS (
            SELECT id FROM "User" WHERE "firebaseUid" = $1
        ), inserted AS (
            INSERT INTO "User" ("firebaseUid", "createdAt")
            SELECT $1, $2 WHERE NOT EXISTS (SELECT 1 FROM existing)
            ON CONFLICT ("firebaseUid") DO NOTHING
            RETURNING id
        )
        SELECT id AS "id!" FROM existing
        UNION ALL
        SELECT id AS "id!" FROM inserted
        "#,
        firebase_uid,
        time::now(),
    )
    .fetch_optional(db)
    .await?;

    if let Some(id) = id {
        return Ok(id);
    }

    // Lost a race with a concurrent first request that inserted the row.
    Ok(sqlx::query_scalar!(
        r#"SELECT id FROM "User" WHERE "firebaseUid" = $1"#,
        firebase_uid
    )
    .fetch_one(db)
    .await?)
}
