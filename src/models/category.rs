use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryWithCount {
    pub id: i32,
    pub name: String,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
    pub user_id: Option<i32>,
    pub expense_count: i64,
}
