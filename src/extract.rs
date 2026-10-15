use axum::extract::{FromRequest, FromRequestParts};
use serde::Deserialize;

use crate::db::{DEFAULT_TAKE, MAX_TAKE};
use crate::error::AppError;

/// JSON body extractor whose rejections use the API's `{ "message": ... }` shape.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct ApiJson<T>(pub T);

/// Numeric `:id` path segment; anything that is not an `i32` is a 400.
#[derive(Deserialize, FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct Id(pub i32);

/// Query string extractor whose rejections use the API's error shape.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct ApiQuery<T>(pub T);

#[derive(Debug, Default, Deserialize)]
pub struct TakeQuery {
    pub take: Option<String>,
}

/// Mirrors `Number(take) || DEFAULT_TAKE`, then clamps to `1..=MAX_TAKE`.
pub fn take_from(raw: Option<&str>) -> i64 {
    let parsed = raw
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .map(|v| v.trunc() as i64)
        .filter(|v| *v != 0)
        .unwrap_or(DEFAULT_TAKE);
    parsed.clamp(1, MAX_TAKE)
}

/// Mirrors the Node `isTrue` helper used for the `i*` include flags.
pub fn is_true(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.eq_ignore_ascii_case("true"))
}

/// Request bodies are read as JSON objects and validated field by field.
pub type Body = serde_json::Map<String, serde_json::Value>;

/// Accepts an array of ids given as numbers or numeric strings, like `Number(id)` in Node.
pub fn parse_id_list(value: Option<&serde_json::Value>) -> Option<Vec<i32>> {
    value?
        .as_array()?
        .iter()
        .map(|item| match item {
            serde_json::Value::Number(n) => n.as_i64().and_then(|v| i32::try_from(v).ok()),
            serde_json::Value::String(s) => s.trim().parse::<i32>().ok(),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn take_defaults_and_clamps() {
        assert_eq!(take_from(None), 10);
        assert_eq!(take_from(Some("abc")), 10);
        assert_eq!(take_from(Some("0")), 10);
        assert_eq!(take_from(Some("25")), 25);
        assert_eq!(take_from(Some("500")), 100);
        assert_eq!(take_from(Some("-5")), 1);
        assert_eq!(take_from(Some("7.9")), 7);
    }

    #[test]
    fn include_flags() {
        assert!(is_true(Some("true")));
        assert!(is_true(Some("TRUE")));
        assert!(!is_true(Some("1")));
        assert!(!is_true(None));
    }

    #[test]
    fn id_lists() {
        assert_eq!(
            parse_id_list(Some(&json!([3, "1", 2]))),
            Some(vec![3, 1, 2])
        );
        assert_eq!(parse_id_list(Some(&json!([1, "x"]))), None);
        assert_eq!(parse_id_list(Some(&json!("1,2"))), None);
        assert_eq!(parse_id_list(None), None);
        assert_eq!(parse_id_list(Some(&json!([]))), Some(vec![]));
    }
}
