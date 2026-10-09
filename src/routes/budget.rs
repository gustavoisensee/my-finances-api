use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::{Postgres, Transaction};

use super::fields;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::extract::{ApiJson, ApiQuery, Body, Id, TakeQuery, parse_id_list, take_from};
use crate::models::Budget;
use crate::state::AppState;

const NAME_MESSAGE: &str = "Name must have at least 3 characters.";
const NOT_FOUND: &str = "Budget not found.";

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<Budget>>> {
    let budgets = sqlx::query_as!(
        Budget,
        r#"
        SELECT b.id, b.description, b.value, b.color, b.index,
               b."createdAt" AS created_at, b."monthId" AS month_id
        FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
        WHERE m."userId" = $1
        ORDER BY b.index, b.id
        LIMIT $2
        "#,
        user.user_id,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(budgets))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<Budget>)> {
    let value = fields::amount(body.get("value"))?;
    let description = fields::min_three_chars(body.get("description"), NAME_MESSAGE)?;
    let color = fields::nullable_text(body.get("color"), "color")?.flatten();
    let month_id =
        fields::month_owned(&state.db, fields::id(body.get("monthId")), user.user_id).await?;
    let created_at = fields::created_at(body.get("createdAt"))?;

    let mut tx = state.db.begin().await?;
    fields::lock_month(&mut *tx, month_id).await?;
    let index = next_index(&mut tx, month_id).await?;

    let budget = sqlx::query_as!(
        Budget,
        r#"
        INSERT INTO "Budget" (description, value, color, index, "createdAt", "monthId")
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, description, value, color, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        description,
        value,
        color,
        index,
        created_at,
        month_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(budget)))
}

/// Moving a budget to another month appends it to the end of that month and
/// closes the gap it left in the old one. An omitted `color` is left unchanged.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Budget>> {
    let value = fields::amount(body.get("value"))?;
    let description = fields::min_three_chars(body.get("description"), NAME_MESSAGE)?;
    let color = fields::nullable_text(body.get("color"), "color")?;
    let month_id =
        fields::month_owned(&state.db, fields::id(body.get("monthId")), user.user_id).await?;

    let mut tx = state.db.begin().await?;
    let current_month_id = sqlx::query_scalar!(
        r#"
        SELECT b."monthId" FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
        WHERE b.id = $1 AND m."userId" = $2
        FOR UPDATE OF b
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found(NOT_FOUND))?;

    let moved = current_month_id != month_id;
    let new_index = if moved {
        fields::lock_month_pair(&mut tx, current_month_id, month_id).await?;
        Some(next_index(&mut tx, month_id).await?)
    } else {
        None
    };

    let budget = sqlx::query_as!(
        Budget,
        r#"
        UPDATE "Budget"
        SET description = $2,
            value = $3,
            color = CASE WHEN $4 THEN $5 ELSE color END,
            "monthId" = $6,
            index = COALESCE($7, index)
        WHERE id = $1
        RETURNING id, description, value, color, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        id,
        description,
        value,
        color.is_some(),
        color.flatten(),
        month_id,
        new_index,
    )
    .fetch_one(&mut *tx)
    .await?;

    if moved {
        reindex(&mut tx, current_month_id).await?;
    }
    tx.commit().await?;
    Ok(Json(budget))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
) -> AppResult<Json<Budget>> {
    let mut tx = state.db.begin().await?;
    let budget = sqlx::query_as!(
        Budget,
        r#"
        SELECT b.id, b.description, b.value, b.color, b.index,
               b."createdAt" AS created_at, b."monthId" AS month_id
        FROM "Budget" b JOIN "Month" m ON m.id = b."monthId"
        WHERE b.id = $1 AND m."userId" = $2
        FOR UPDATE OF b
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found(NOT_FOUND))?;

    fields::lock_month(&mut *tx, budget.month_id).await?;
    sqlx::query!(r#"DELETE FROM "Budget" WHERE id = $1"#, id)
        .execute(&mut *tx)
        .await?;
    reindex(&mut tx, budget.month_id).await?;
    tx.commit().await?;

    Ok(Json(budget))
}

pub async fn reorder(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Vec<Budget>>> {
    let required = || AppError::validation("monthId and budgetIds array are required.");
    let month_id = fields::id(body.get("monthId"))
        .filter(|v| *v != 0)
        .ok_or_else(required)?;
    let budget_ids = parse_id_list(body.get("budgetIds")).ok_or_else(required)?;
    let month_id = fields::month_owned(&state.db, Some(month_id), user.user_id).await?;

    let mut tx = state.db.begin().await?;
    fields::lock_month(&mut *tx, month_id).await?;

    let found = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM "Budget" WHERE id = ANY($1) AND "monthId" = $2"#,
        &budget_ids,
        month_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if found != budget_ids.len() as i64 {
        return Err(AppError::validation(
            "Some budgets do not exist or do not belong to this month.",
        ));
    }

    sqlx::query!(
        r#"
        UPDATE "Budget" b SET index = (t.ord - 1)::INT
        FROM unnest($1::INT[]) WITH ORDINALITY AS t(id, ord)
        WHERE b.id = t.id
        "#,
        &budget_ids,
    )
    .execute(&mut *tx)
    .await?;

    let budgets = sqlx::query_as!(
        Budget,
        r#"
        SELECT id, description, value, color, index, "createdAt" AS created_at, "monthId" AS month_id
        FROM "Budget" WHERE "monthId" = $1
        ORDER BY index, id
        "#,
        month_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(budgets))
}

async fn next_index(tx: &mut Transaction<'_, Postgres>, month_id: i32) -> AppResult<i32> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(index) + 1, 0) AS "index!" FROM "Budget" WHERE "monthId" = $1"#,
        month_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// Renumbers a month's budgets to `0..n` keeping their relative order.
async fn reindex(tx: &mut Transaction<'_, Postgres>, month_id: i32) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE "Budget" b SET index = r.new_index
        FROM (
            SELECT id, (ROW_NUMBER() OVER (ORDER BY index, id) - 1)::INT AS new_index
            FROM "Budget" WHERE "monthId" = $1
        ) r
        WHERE b.id = r.id AND b.index <> r.new_index
        "#,
        month_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
