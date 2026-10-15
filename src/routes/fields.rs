//! Validation helpers for request bodies. Fields are read as loose JSON values so
//! wrong types produce the same messages the Node API returned.

use chrono::NaiveDateTime;
use serde_json::Value;
use sqlx::PgExecutor;

use crate::error::{AppError, AppResult};
use crate::time;

pub const AMOUNT_MESSAGE: &str = "Value must be a number and greater than 0.";
pub const MONTH_VALUE_MESSAGE: &str = "Value must be a number between 1 and 12.";
pub const MONTH_NOT_FOUND: &str = "Month does not exist.";

/// `isNaN(value) || value <= 0`
pub fn amount(value: Option<&Value>) -> AppResult<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite() && *v > 0.0)
        .ok_or_else(|| AppError::validation(AMOUNT_MESSAGE))
}

/// `Number.isInteger(value) && value >= 1 && value <= 12`
pub fn month_value(value: Option<&Value>) -> AppResult<i32> {
    value
        .and_then(Value::as_f64)
        .filter(|v| v.fract() == 0.0 && (1.0..=12.0).contains(v))
        .map(|v| v as i32)
        .ok_or_else(|| AppError::validation(MONTH_VALUE_MESSAGE))
}

/// A string with at least three characters.
pub fn min_three_chars(value: Option<&Value>, message: &str) -> AppResult<String> {
    value
        .and_then(Value::as_str)
        .filter(|s| s.chars().count() >= 3)
        .map(str::to_owned)
        .ok_or_else(|| AppError::validation(message))
}

/// Ids accept numbers or numeric strings, like `Number(id)` did.
pub fn id(value: Option<&Value>) -> Option<i32> {
    match value? {
        Value::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

pub fn created_at(value: Option<&Value>) -> AppResult<NaiveDateTime> {
    value
        .and_then(Value::as_str)
        .and_then(time::parse)
        .ok_or_else(|| AppError::validation("createdAt must be a valid ISO-8601 date-time."))
}

/// For nullable text columns. `None` = field absent (leave unchanged),
/// `Some(None)` = explicit `null`.
pub fn nullable_text(value: Option<&Value>, field: &str) -> AppResult<Option<Option<String>>> {
    match value {
        None => Ok(None),
        Some(Value::Null) => Ok(Some(None)),
        Some(Value::String(s)) => Ok(Some(Some(s.clone()))),
        Some(_) => Err(AppError::validation(format!("{field} must be a string."))),
    }
}

pub async fn month_owned<'e>(
    db: impl PgExecutor<'e>,
    month_id: Option<i32>,
    user_id: i32,
) -> AppResult<i32> {
    let not_found = || AppError::not_found(MONTH_NOT_FOUND);
    let month_id = month_id.ok_or_else(not_found)?;
    sqlx::query_scalar!(
        r#"SELECT id FROM "Month" WHERE id = $1 AND "userId" = $2"#,
        month_id,
        user_id,
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(not_found)
}

/// Locks the month row so concurrent index assignment within it is serialized.
pub async fn lock_month<'e>(db: impl PgExecutor<'e>, month_id: i32) -> AppResult<()> {
    sqlx::query!(
        r#"SELECT id FROM "Month" WHERE id = $1 FOR UPDATE"#,
        month_id
    )
    .fetch_optional(db)
    .await?;
    Ok(())
}

/// Locks two months in ascending id order so concurrent moves cannot deadlock.
pub async fn lock_month_pair(conn: &mut sqlx::PgConnection, a: i32, b: i32) -> AppResult<()> {
    let (first, second) = if a <= b { (a, b) } else { (b, a) };
    lock_month(&mut *conn, first).await?;
    lock_month(&mut *conn, second).await
}

pub async fn year_exists<'e>(db: impl PgExecutor<'e>, year_id: i32) -> AppResult<()> {
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM "Year" WHERE id = $1) AS "exists!""#,
        year_id
    )
    .fetch_one(db)
    .await?;
    if exists {
        Ok(())
    } else {
        Err(AppError::validation("Year does not exist."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn amounts() {
        assert_eq!(amount(Some(&json!(85.5))).unwrap(), 85.5);
        assert!(amount(Some(&json!(0))).is_err());
        assert!(amount(Some(&json!(-1))).is_err());
        assert!(amount(Some(&json!("5"))).is_err());
        assert!(amount(None).is_err());
    }

    #[test]
    fn month_values() {
        assert_eq!(month_value(Some(&json!(1))).unwrap(), 1);
        assert_eq!(month_value(Some(&json!(12.0))).unwrap(), 12);
        assert!(month_value(Some(&json!(13))).is_err());
        assert!(month_value(Some(&json!(0))).is_err());
        assert!(month_value(Some(&json!(1.5))).is_err());
        assert!(month_value(Some(&json!("3"))).is_err());
    }

    #[test]
    fn texts_and_ids() {
        assert!(min_three_chars(Some(&json!("abc")), "m").is_ok());
        assert!(min_three_chars(Some(&json!("ab")), "m").is_err());
        assert!(min_three_chars(Some(&json!(12345)), "m").is_err());
        assert_eq!(id(Some(&json!(4))), Some(4));
        assert_eq!(id(Some(&json!("4"))), Some(4));
        assert_eq!(id(Some(&json!("x"))), None);
        assert_eq!(nullable_text(None, "f").unwrap(), None);
        assert_eq!(nullable_text(Some(&json!(null)), "f").unwrap(), Some(None));
        assert!(nullable_text(Some(&json!(1)), "f").is_err());
    }
}
