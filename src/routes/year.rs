use axum::Json;
use axum::extract::State;

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::extract::{ApiQuery, TakeQuery, take_from};
use crate::models::Year;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    _user: AuthUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<Year>>> {
    let years = sqlx::query_as!(
        Year,
        r#"SELECT id, value, "createdAt" AS created_at FROM "Year" ORDER BY id LIMIT $1"#,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(years))
}
