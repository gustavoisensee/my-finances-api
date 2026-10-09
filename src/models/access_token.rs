use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessToken {
    pub id: i32,
    pub name: String,
    pub value: String,
    #[serde(serialize_with = "crate::time::serialize")]
    pub expiration_date: NaiveDateTime,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
}
