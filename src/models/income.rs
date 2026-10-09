use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Income {
    pub id: i32,
    pub description: Option<String>,
    pub value: f64,
    pub index: i32,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
    pub month_id: i32,
}
