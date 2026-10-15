use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use sqlx::{Postgres, Transaction};

use super::fields;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::extract::{ApiJson, ApiQuery, Body, Id, TakeQuery, parse_id_list, take_from};
use crate::models::Income;
use crate::state::AppState;

const NOT_FOUND: &str = "Income not found.";

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<Income>>> {
    let incomes = sqlx::query_as!(
        Income,
        r#"
        SELECT i.id, i.description, i.value, i.index,
               i."createdAt" AS created_at, i."monthId" AS month_id
        FROM "Income" i JOIN "Month" m ON m.id = i."monthId"
        WHERE m."userId" = $1
        ORDER BY i.index, i.id
        LIMIT $2
        "#,
        user.user_id,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(incomes))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<Income>)> {
    let value = fields::amount(body.get("value"))?;
    let description = fields::nullable_text(body.get("description"), "description")?.flatten();
    let month_id =
        fields::month_owned(&state.db, fields::id(body.get("monthId")), user.user_id).await?;
    let created_at = fields::created_at(body.get("createdAt"))?;

    let mut tx = state.db.begin().await?;
    fields::lock_month(&mut *tx, month_id).await?;
    let index = next_index(&mut tx, month_id).await?;

    let income = sqlx::query_as!(
        Income,
        r#"
        INSERT INTO "Income" (description, value, index, "createdAt", "monthId")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, description, value, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        description,
        value,
        index,
        created_at,
        month_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(income)))
}

/// Moving an income to another month appends it to the end of that month and
/// closes the gap it left in the old one. An omitted `description` is left unchanged.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Income>> {
    let value = fields::amount(body.get("value"))?;
    let description = fields::nullable_text(body.get("description"), "description")?;
    let month_id =
        fields::month_owned(&state.db, fields::id(body.get("monthId")), user.user_id).await?;

    let mut tx = state.db.begin().await?;
    let current_month_id = sqlx::query_scalar!(
        r#"
        SELECT i."monthId" FROM "Income" i JOIN "Month" m ON m.id = i."monthId"
        WHERE i.id = $1 AND m."userId" = $2
        FOR UPDATE OF i
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

    let income = sqlx::query_as!(
        Income,
        r#"
        UPDATE "Income"
        SET value = $2,
            description = CASE WHEN $3 THEN $4 ELSE description END,
            "monthId" = $5,
            index = COALESCE($6, index)
        WHERE id = $1
        RETURNING id, description, value, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        id,
        value,
        description.is_some(),
        description.flatten(),
        month_id,
        new_index,
    )
    .fetch_one(&mut *tx)
    .await?;

    if moved {
        reindex(&mut tx, current_month_id).await?;
    }
    tx.commit().await?;
    Ok(Json(income))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
) -> AppResult<Json<Income>> {
    let mut tx = state.db.begin().await?;
    let income = sqlx::query_as!(
        Income,
        r#"
        SELECT i.id, i.description, i.value, i.index,
               i."createdAt" AS created_at, i."monthId" AS month_id
        FROM "Income" i JOIN "Month" m ON m.id = i."monthId"
        WHERE i.id = $1 AND m."userId" = $2
        FOR UPDATE OF i
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found(NOT_FOUND))?;

    fields::lock_month(&mut *tx, income.month_id).await?;
    sqlx::query!(r#"DELETE FROM "Income" WHERE id = $1"#, id)
        .execute(&mut *tx)
        .await?;
    reindex(&mut tx, income.month_id).await?;
    tx.commit().await?;

    Ok(Json(income))
}

pub async fn reorder(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Vec<Income>>> {
    let required = || AppError::validation("monthId and incomeIds array are required.");
    let month_id = fields::id(body.get("monthId"))
        .filter(|v| *v != 0)
        .ok_or_else(required)?;
    let income_ids = parse_id_list(body.get("incomeIds")).ok_or_else(required)?;
    let month_id = fields::month_owned(&state.db, Some(month_id), user.user_id).await?;

    let mut tx = state.db.begin().await?;
    fields::lock_month(&mut *tx, month_id).await?;

    let found = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM "Income" WHERE id = ANY($1) AND "monthId" = $2"#,
        &income_ids,
        month_id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if found != income_ids.len() as i64 {
        return Err(AppError::validation(
            "Some incomes do not exist or do not belong to this month.",
        ));
    }

    sqlx::query!(
        r#"
        UPDATE "Income" i SET index = (t.ord - 1)::INT
        FROM unnest($1::INT[]) WITH ORDINALITY AS t(id, ord)
        WHERE i.id = t.id
        "#,
        &income_ids,
    )
    .execute(&mut *tx)
    .await?;

    let incomes = sqlx::query_as!(
        Income,
        r#"
        SELECT id, description, value, index, "createdAt" AS created_at, "monthId" AS month_id
        FROM "Income" WHERE "monthId" = $1
        ORDER BY index, id
        "#,
        month_id,
    )
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(incomes))
}

async fn next_index(tx: &mut Transaction<'_, Postgres>, month_id: i32) -> AppResult<i32> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(index) + 1, 0) AS "index!" FROM "Income" WHERE "monthId" = $1"#,
        month_id,
    )
    .fetch_one(&mut **tx)
    .await?)
}

/// Renumbers a month's incomes to `0..n` keeping their relative order.
async fn reindex(tx: &mut Transaction<'_, Postgres>, month_id: i32) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE "Income" i SET index = r.new_index
        FROM (
            SELECT id, (ROW_NUMBER() OVER (ORDER BY index, id) - 1)::INT AS new_index
            FROM "Income" WHERE "monthId" = $1
        ) r
        WHERE i.id = r.id AND i.index <> r.new_index
        "#,
        month_id,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
