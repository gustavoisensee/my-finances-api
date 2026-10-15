use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::extract::{ApiJson, Body, Id};
use crate::models::CategoryWithCount;
use crate::state::AppState;
use crate::time;

fn parse_name(body: &Body) -> AppResult<String> {
    body.get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AppError::validation("Name is required."))
}

#[derive(Debug, Serialize)]
pub struct DeleteResult {
    success: bool,
    message: &'static str,
}

/// The user's own categories plus the global defaults (`userId IS NULL`).
/// `expenseCount` only counts the requesting user's expenses so usage of global
/// categories by other users is not disclosed.
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<CategoryWithCount>>> {
    let categories = sqlx::query_as!(
        CategoryWithCount,
        r#"
        SELECT c.id, c.name, c."createdAt" AS created_at, c."userId" AS user_id,
               (SELECT COUNT(*) FROM "Expense" e
                  JOIN "Budget" b ON b.id = e."budgetId"
                  JOIN "Month" m ON m.id = b."monthId"
                 WHERE e."categoryId" = c.id AND m."userId" = $1) AS "expense_count!"
        FROM "Category" c
        WHERE c."userId" = $1 OR c."userId" IS NULL
        ORDER BY c.id
        "#,
        user.user_id,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(categories))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<CategoryWithCount>)> {
    let name = parse_name(&body)?;
    let mut tx = state.db.begin().await?;
    lock_user_categories(&mut tx, user.user_id).await?;

    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM "Category" WHERE name = $1 AND "userId" = $2) AS "exists!""#,
        name,
        user.user_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if exists {
        return Err(AppError::validation("Category name must be unique."));
    }

    let category = sqlx::query_as!(
        CategoryWithCount,
        r#"
        INSERT INTO "Category" (name, "userId", "createdAt")
        VALUES ($1, $2, $3)
        RETURNING id, name, "createdAt" AS created_at, "userId" AS user_id, 0::BIGINT AS "expense_count!"
        "#,
        name,
        user.user_id,
        time::now(),
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(category)))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<CategoryWithCount>> {
    let name = parse_name(&body)?;
    let mut tx = state.db.begin().await?;
    lock_user_categories(&mut tx, user.user_id).await?;

    let owned = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM "Category" WHERE id = $1 AND "userId" = $2) AS "exists!""#,
        id,
        user.user_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if !owned {
        return Err(AppError::not_found(
            "Category not found or you do not have permission to edit it.",
        ));
    }

    let duplicate = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM "Category" WHERE name = $1 AND "userId" = $2 AND id <> $3) AS "exists!""#,
        name,
        user.user_id,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if duplicate {
        return Err(AppError::validation("Category name must be unique."));
    }

    let category = sqlx::query_as!(
        CategoryWithCount,
        r#"
        UPDATE "Category" c SET name = $2
        WHERE c.id = $1
        RETURNING c.id, c.name, c."createdAt" AS created_at, c."userId" AS user_id,
                  (SELECT COUNT(*) FROM "Expense" e
                     JOIN "Budget" b ON b.id = e."budgetId"
                     JOIN "Month" m ON m.id = b."monthId"
                    WHERE e."categoryId" = c.id AND m."userId" = $3) AS "expense_count!"
        "#,
        id,
        name,
        user.user_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(category))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
) -> AppResult<Json<DeleteResult>> {
    let mut tx = state.db.begin().await?;

    let owned = sqlx::query_scalar!(
        r#"SELECT id FROM "Category" WHERE id = $1 AND "userId" = $2 FOR UPDATE"#,
        id,
        user.user_id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    if owned.is_none() {
        return Err(AppError::not_found(
            "Category not found or you do not have permission to delete it.",
        ));
    }

    let expense_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM "Expense" WHERE "categoryId" = $1"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if expense_count > 0 {
        let plural = if expense_count > 1 { "s" } else { "" };
        return Err(AppError::validation(format!(
            "Cannot delete category: it is used by {expense_count} expense{plural}."
        )));
    }

    sqlx::query!(r#"DELETE FROM "Category" WHERE id = $1"#, id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    Ok(Json(DeleteResult {
        success: true,
        message: "Category deleted successfully",
    }))
}

/// Serializes category writes per user so the name-uniqueness check cannot race.
async fn lock_user_categories(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: i32,
) -> AppResult<()> {
    sqlx::query!(r#"SELECT id FROM "User" WHERE id = $1 FOR UPDATE"#, user_id)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(())
}
