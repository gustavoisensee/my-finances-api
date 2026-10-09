use chrono::NaiveDateTime;
use serde::Serialize;

use super::{BudgetWithExpenses, Income};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Month {
    pub id: i32,
    pub value: i32,
    pub description: String,
    #[serde(serialize_with = "crate::time::serialize")]
    pub created_at: NaiveDateTime,
    pub year_id: i32,
    pub user_id: i32,
}

/// A month with optional relations; relations that were not requested are
/// omitted from the JSON, like Prisma's `include: false`.
#[derive(Debug, Clone, Serialize)]
pub struct MonthWithRelations {
    #[serde(flatten)]
    pub month: Month,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomes: Option<Vec<Income>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budgets: Option<Vec<BudgetWithExpenses>>,
}
