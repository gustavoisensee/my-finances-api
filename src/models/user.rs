use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub firebase_uid: Option<String>,
    pub clerk_id: Option<String>,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
}
