use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Year {
    pub id: i32,
    pub value: i32,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
}
