use crate::auth::AdminUser;
use crate::error::{AppError, AppResult, is_foreign_key_violation, is_unique_violation};
use crate::extract::{ApiJson, ApiQuery, Body, Id, TakeQuery, take_from};
use crate::models::User;
use crate::state::AppState;
use crate::time;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

fn parse_firebase_uid(body: &Body) -> AppResult<String> {
    body.get("firebaseUid")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|uid| !uid.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AppError::validation("firebaseUid is required."))
}

fn map_write_error(err: sqlx::Error) -> AppError {
    if is_unique_violation(&err) {
        AppError::validation("firebaseUid is already in use.")
    } else {
        err.into()
    }
}

pub async fn list(
    State(state): State<AppState>,
    _admin: AdminUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<User>>> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, "firebaseUid" AS firebase_uid, "clerkId" AS clerk_id, "createdAt" AS created_at
        FROM "User"
        ORDER BY id
        LIMIT $1
        "#,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(users))
}

pub async fn create(
    State(state): State<AppState>,
    _admin: AdminUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<User>)> {
    let firebase_uid = parse_firebase_uid(&body)?;
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO "User" ("firebaseUid", "createdAt")
        VALUES ($1, $2)
        RETURNING id, "firebaseUid" AS firebase_uid, "clerkId" AS clerk_id, "createdAt" AS created_at
        "#,
        firebase_uid,
        time::now(),
    )
    .fetch_one(&state.db)
    .await
    .map_err(map_write_error)?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update(
    State(state): State<AppState>,
    _admin: AdminUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<User>> {
    let firebase_uid = parse_firebase_uid(&body)?;
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE "User" SET "firebaseUid" = $2
        WHERE id = $1
        RETURNING id, "firebaseUid" AS firebase_uid, "clerkId" AS clerk_id, "createdAt" AS created_at
        "#,
        id,
        firebase_uid,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(map_write_error)?
    .ok_or_else(|| AppError::not_found("User not found."))?;
    Ok(Json(user))
}

/// Deletes the user together with all of their data.
pub async fn remove(
    State(state): State<AppState>,
    _admin: AdminUser,
    Id(id): Id,
) -> AppResult<Json<User>> {
    let mut tx = state.db.begin().await?;

    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, "firebaseUid" AS firebase_uid, "clerkId" AS clerk_id, "createdAt" AS created_at
        FROM "User" WHERE id = $1
        FOR UPDATE
        "#,
        id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("User not found."))?;

    // Incomes, budgets and their expenses are removed by ON DELETE CASCADE.
    sqlx::query!(r#"DELETE FROM "Month" WHERE "userId" = $1"#, id)
        .execute(&mut *tx)
        .await?;

    sqlx::query!(r#"DELETE FROM "Category" WHERE "userId" = $1"#, id)
        .execute(&mut *tx)
        .await
        .map_err(|err| {
            if is_foreign_key_violation(&err) {
                AppError::validation(
                    "Cannot delete user: some of their categories are used by other users' expenses.",
                )
            } else {
                err.into()
            }
        })?;

    sqlx::query!(r#"DELETE FROM "User" WHERE id = $1"#, id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(Json(user))
}
