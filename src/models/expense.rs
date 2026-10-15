use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: i32,
    pub description: String,
    pub value: f64,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
    pub budget_id: i32,
    pub category_id: i32,
}
