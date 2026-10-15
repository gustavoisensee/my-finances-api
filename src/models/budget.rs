use chrono::NaiveDateTime;
use serde::Serialize;

use super::Expense;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    pub id: i32,
    pub description: String,
    pub value: f64,
    pub color: Option<String>,
    pub index: i32,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
    pub month_id: i32,
}

/// A budget as returned inside a month; `expenses` is only present when requested.
#[derive(Debug, Clone, Serialize)]
pub struct BudgetWithExpenses {
    #[serde(flatten)]
    pub budget: Budget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expenses: Option<Vec<Expense>>,
}
