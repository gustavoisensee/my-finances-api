use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::PgPool;

use super::fields;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::extract::{ApiJson, ApiQuery, Body, Id, TakeQuery, take_from};
use crate::models::Expense;
use crate::state::AppState;

const TITLE_MESSAGE: &str = "Title must have at least 3 characters.";
const NOT_FOUND: &str = "Expense not found.";

struct ExpenseInput {
    value: f64,
    description: String,
    budget_id: i32,
    category_id: i32,
}

/// Shared create/update validation: the budget must belong to the user and the
/// category must be one of the user's own or a global default.
async fn validate(db: &PgPool, body: &Body, user_id: i32) -> AppResult<ExpenseInput> {
    let value = fields::amount(body.get("value"))?;
    let description = fields::min_three_chars(body.get("description"), TITLE_MESSAGE)?;

    let budget_not_found = || AppError::not_found("Budget does not exist.");
    let budget_id = fields::id(body.get("budgetId")).ok_or_else(budget_not_found)?;
    let budget_owned = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
            WHERE b.id = $1 AND m."userId" = $2
        ) AS "exists!"
        "#,
        budget_id,
        user_id,
    )
    .fetch_one(db)
    .await?;
    if !budget_owned {
        return Err(budget_not_found());
    }

    let category_not_found = || AppError::not_found("Category does not exist.");
    let category_id = fields::id(body.get("categoryId")).ok_or_else(category_not_found)?;
    let category_allowed = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM "Category"
            WHERE id = $1 AND ("userId" = $2 OR "userId" IS NULL)
        ) AS "exists!"
        "#,
        category_id,
        user_id,
    )
    .fetch_one(db)
    .await?;
    if !category_allowed {
        return Err(category_not_found());
    }

    Ok(ExpenseInput {
        value,
        description,
        budget_id,
        category_id,
    })
}

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<Expense>>> {
    let expenses = sqlx::query_as!(
        Expense,
        r#"
        SELECT e.id, e.description, e.value, e."createdAt" AS created_at,
               e."budgetId" AS budget_id, e."categoryId" AS category_id
        FROM "Expense" e
        JOIN "Budget" b ON b.id = e."budgetId"
        JOIN "Month" m ON m.id = b."monthId"
        WHERE m."userId" = $1
        ORDER BY e.id
        LIMIT $2
        "#,
        user.user_id,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(expenses))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<Expense>)> {
    let input = validate(&state.db, &body, user.user_id).await?;
    let created_at = fields::created_at(body.get("createdAt"))?;

    let expense = sqlx::query_as!(
        Expense,
        r#"
        INSERT INTO "Expense" (value, description, "createdAt", "budgetId", "categoryId")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, description, value, "createdAt" AS created_at,
                  "budgetId" AS budget_id, "categoryId" AS category_id
        "#,
        input.value,
        input.description,
        created_at,
        input.budget_id,
        input.category_id,
    )
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(expense)))
}

pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Expense>> {
    let input = validate(&state.db, &body, user.user_id).await?;

    let expense = sqlx::query_as!(
        Expense,
        r#"
        UPDATE "Expense" e
        SET value = $3, description = $4, "budgetId" = $5, "categoryId" = $6
        FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
        WHERE e.id = $1 AND b.id = e."budgetId" AND m."userId" = $2
        RETURNING e.id, e.description, e.value, e."createdAt" AS created_at,
                  e."budgetId" AS budget_id, e."categoryId" AS category_id
        "#,
        id,
        user.user_id,
        input.value,
        input.description,
        input.budget_id,
        input.category_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found(NOT_FOUND))?;
    Ok(Json(expense))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
) -> AppResult<Json<Expense>> {
    let expense = sqlx::query_as!(
        Expense,
        r#"
        DELETE FROM "Expense" e
        USING "Budget" b, "Month" m
        WHERE e.id = $1 AND b.id = e."budgetId" AND m.id = b."monthId" AND m."userId" = $2
        RETURNING e.id, e.description, e.value, e."createdAt" AS created_at,
                  e."budgetId" AS budget_id, e."categoryId" AS category_id
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found(NOT_FOUND))?;
    Ok(Json(expense))
}
