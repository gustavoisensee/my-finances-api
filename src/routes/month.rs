use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;

use super::fields;
use crate::auth::AuthUser;
use crate::error::{AppError, AppResult};
use crate::extract::{ApiJson, ApiQuery, Body, Id, is_true, take_from};
use crate::models::{Budget, BudgetWithExpenses, Expense, Income, Month, MonthWithRelations};
use crate::state::AppState;
use crate::time;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncludeQuery {
    pub take: Option<String>,
    pub year_id: Option<String>,
    pub i_incomes: Option<String>,
    pub i_budgets: Option<String>,
    pub i_expenses: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct Includes {
    incomes: bool,
    budgets: bool,
    expenses: bool,
}

impl From<&IncludeQuery> for Includes {
    fn from(q: &IncludeQuery) -> Self {
        Self {
            incomes: is_true(q.i_incomes.as_deref()),
            budgets: is_true(q.i_budgets.as_deref()),
            expenses: is_true(q.i_expenses.as_deref()),
        }
    }
}

fn month_name(value: i32) -> &'static str {
    usize::try_from(value - 1)
        .ok()
        .and_then(|i| MONTH_NAMES.get(i))
        .copied()
        .unwrap_or("Unknown")
}

pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<IncludeQuery>,
) -> AppResult<Json<Vec<MonthWithRelations>>> {
    let year_id = query
        .year_id
        .as_deref()
        .and_then(|v| v.trim().parse::<i32>().ok())
        .filter(|v| *v != 0)
        .ok_or_else(|| AppError::validation("yearId must be passed."))?;

    let months = sqlx::query_as!(
        Month,
        r#"
        SELECT id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        FROM "Month"
        WHERE "userId" = $1 AND "yearId" = $2
        ORDER BY id
        LIMIT $3
        "#,
        user.user_id,
        year_id,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(
        with_relations(&state.db, months, Includes::from(&query)).await?,
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiQuery(query): ApiQuery<IncludeQuery>,
) -> AppResult<Json<MonthWithRelations>> {
    let month = sqlx::query_as!(
        Month,
        r#"
        SELECT id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        FROM "Month"
        WHERE id = $1 AND "userId" = $2
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found("Month not found."))?;

    let mut months = with_relations(&state.db, vec![month], Includes::from(&query)).await?;
    Ok(Json(months.remove(0)))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<Month>)> {
    let value = fields::month_value(body.get("value"))?;
    let description = body
        .get("description")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::validation("description is required."))?;
    let created_at = fields::created_at(body.get("createdAt"))?;
    let year_id = fields::id(body.get("yearId"))
        .ok_or_else(|| AppError::validation("yearId is required."))?;
    fields::year_exists(&state.db, year_id).await?;

    let month = sqlx::query_as!(
        Month,
        r#"
        INSERT INTO "Month" (value, description, "createdAt", "yearId", "userId")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        "#,
        value,
        description,
        created_at,
        year_id,
        user.user_id,
    )
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(month)))
}

/// `description` and `yearId` are optional; omitted fields keep their values.
pub async fn update(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<Json<Month>> {
    let value = fields::month_value(body.get("value"))?;
    let description = match body.get("description") {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(_) => return Err(AppError::validation("description must be a string.")),
    };
    let year_id = match body.get("yearId") {
        None | Some(serde_json::Value::Null) => None,
        raw => Some(fields::id(raw).ok_or_else(|| AppError::validation("Year does not exist."))?),
    };
    if let Some(year_id) = year_id {
        fields::year_exists(&state.db, year_id).await?;
    }

    let month = sqlx::query_as!(
        Month,
        r#"
        UPDATE "Month"
        SET value = $3,
            description = COALESCE($4, description),
            "yearId" = COALESCE($5, "yearId")
        WHERE id = $1 AND "userId" = $2
        RETURNING id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        "#,
        id,
        user.user_id,
        value,
        description,
        year_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found("Month not found."))?;
    Ok(Json(month))
}

pub async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
    Id(id): Id,
) -> AppResult<Json<Month>> {
    let month = sqlx::query_as!(
        Month,
        r#"
        DELETE FROM "Month"
        WHERE id = $1 AND "userId" = $2
        RETURNING id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        "#,
        id,
        user.user_id,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found("Month not found."))?;
    Ok(Json(month))
}

/// Copies a month's incomes and budgets (not expenses) into a new month.
pub async fn copy(
    State(state): State<AppState>,
    user: AuthUser,
    Id(source_id): Id,
    ApiJson(body): ApiJson<Body>,
) -> AppResult<(StatusCode, Json<MonthWithRelations>)> {
    let value = fields::month_value(body.get("value"))?;
    let year_id = fields::id(body.get("yearId"))
        .filter(|v| *v != 0)
        .ok_or_else(|| AppError::validation("yearId is required."))?;
    fields::year_exists(&state.db, year_id).await?;

    let mut tx = state.db.begin().await?;

    let source = sqlx::query!(
        r#"
        SELECT m.value, y.value AS year_value
        FROM "Month" m JOIN "Year" y ON y.id = m."yearId"
        WHERE m.id = $1 AND m."userId" = $2
        "#,
        source_id,
        user.user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::not_found("Source month not found."))?;

    let description = format!(
        "Copy from {} {}",
        month_name(source.value),
        source.year_value
    );
    let now = time::now();

    let month = sqlx::query_as!(
        Month,
        r#"
        INSERT INTO "Month" (value, description, "createdAt", "yearId", "userId")
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, value, description, "createdAt" AS created_at, "yearId" AS year_id, "userId" AS user_id
        "#,
        value,
        description,
        now,
        year_id,
        user.user_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    let mut incomes = sqlx::query_as!(
        Income,
        r#"
        INSERT INTO "Income" (description, value, index, "createdAt", "monthId")
        SELECT description, value, index, $3, $2 FROM "Income" WHERE "monthId" = $1
        RETURNING id, description, value, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        source_id,
        month.id,
        now,
    )
    .fetch_all(&mut *tx)
    .await?;
    incomes.sort_by_key(|i| (i.index, i.id));

    let mut budgets = sqlx::query_as!(
        Budget,
        r#"
        INSERT INTO "Budget" (description, value, color, index, "createdAt", "monthId")
        SELECT description, value, color, index, $3, $2 FROM "Budget" WHERE "monthId" = $1
        RETURNING id, description, value, color, index, "createdAt" AS created_at, "monthId" AS month_id
        "#,
        source_id,
        month.id,
        now,
    )
    .fetch_all(&mut *tx)
    .await?;
    budgets.sort_by_key(|b| (b.index, b.id));

    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(MonthWithRelations {
            month,
            incomes: Some(incomes),
            budgets: Some(
                budgets
                    .into_iter()
                    .map(|budget| BudgetWithExpenses {
                        budget,
                        expenses: None,
                    })
                    .collect(),
            ),
        }),
    ))
}

async fn with_relations(
    db: &PgPool,
    months: Vec<Month>,
    includes: Includes,
) -> AppResult<Vec<MonthWithRelations>> {
    let month_ids: Vec<i32> = months.iter().map(|m| m.id).collect();

    let mut incomes = HashMap::<i32, Vec<Income>>::new();
    if includes.incomes && !month_ids.is_empty() {
        let rows = sqlx::query_as!(
            Income,
            r#"
            SELECT id, description, value, index, "createdAt" AS created_at, "monthId" AS month_id
            FROM "Income" WHERE "monthId" = ANY($1)
            ORDER BY index, id
            "#,
            &month_ids,
        )
        .fetch_all(db)
        .await?;
        for income in rows {
            incomes.entry(income.month_id).or_default().push(income);
        }
    }

    let mut budgets = HashMap::<i32, Vec<BudgetWithExpenses>>::new();
    if includes.budgets && !month_ids.is_empty() {
        let rows = sqlx::query_as!(
            Budget,
            r#"
            SELECT id, description, value, color, index, "createdAt" AS created_at, "monthId" AS month_id
            FROM "Budget" WHERE "monthId" = ANY($1)
            ORDER BY index, id
            "#,
            &month_ids,
        )
        .fetch_all(db)
        .await?;

        let mut expenses = HashMap::<i32, Vec<Expense>>::new();
        if includes.expenses && !rows.is_empty() {
            let budget_ids: Vec<i32> = rows.iter().map(|b| b.id).collect();
            let expense_rows = sqlx::query_as!(
                Expense,
                r#"
                SELECT id, description, value, "createdAt" AS created_at,
                       "budgetId" AS budget_id, "categoryId" AS category_id
                FROM "Expense" WHERE "budgetId" = ANY($1)
                ORDER BY id
                "#,
                &budget_ids,
            )
            .fetch_all(db)
            .await?;
            for expense in expense_rows {
                expenses.entry(expense.budget_id).or_default().push(expense);
            }
        }

        for budget in rows {
            let budget_expenses = includes
                .expenses
                .then(|| expenses.remove(&budget.id).unwrap_or_default());
            budgets
                .entry(budget.month_id)
                .or_default()
                .push(BudgetWithExpenses {
                    budget,
                    expenses: budget_expenses,
                });
        }
    }

    Ok(months
        .into_iter()
        .map(|month| MonthWithRelations {
            incomes: includes
                .incomes
                .then(|| incomes.remove(&month.id).unwrap_or_default()),
            budgets: includes
                .budgets
                .then(|| budgets.remove(&month.id).unwrap_or_default()),
            month,
        })
        .collect())
}
