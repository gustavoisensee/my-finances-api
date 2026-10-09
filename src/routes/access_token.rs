use axum::Json;
use axum::extract::State;

use crate::auth::AdminUser;
use crate::error::AppResult;
use crate::extract::{ApiQuery, TakeQuery, take_from};
use crate::models::AccessToken;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    _admin: AdminUser,
    ApiQuery(query): ApiQuery<TakeQuery>,
) -> AppResult<Json<Vec<AccessToken>>> {
    let tokens = sqlx::query_as!(
        AccessToken,
        r#"
        SELECT id, name, value,
               "expirationDate" AS expiration_date,
               "createdAt" AS created_at
        FROM "AccessToken"
        ORDER BY id
        LIMIT $1
        "#,
        take_from(query.take.as_deref()),
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(tokens))
}
