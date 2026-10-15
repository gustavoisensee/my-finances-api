//! Removes all rows from every table (the schema is kept). Refuses to run in production.

use std::env;

use my_finances_api::config::is_production_env;
use my_finances_api::state::BoxError;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    dotenvy::dotenv().ok();

    if is_production_env(
        env::var("APP_ENV").ok().as_deref(),
        env::var("VERCEL_ENV").ok().as_deref(),
    ) {
        return Err("refusing to clean the database in production".into());
    }

    let url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let mut tx = pool.begin().await?;

    println!("Starting database cleanup...\n");

    // Most dependent tables first so foreign keys are respected.
    let expenses = sqlx::query!(r#"DELETE FROM "Expense""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} expenses", expenses.rows_affected());
    let budgets = sqlx::query!(r#"DELETE FROM "Budget""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} budgets", budgets.rows_affected());
    let incomes = sqlx::query!(r#"DELETE FROM "Income""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} incomes", incomes.rows_affected());
    let months = sqlx::query!(r#"DELETE FROM "Month""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} months", months.rows_affected());
    let categories = sqlx::query!(r#"DELETE FROM "Category""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} categories", categories.rows_affected());
    let years = sqlx::query!(r#"DELETE FROM "Year""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} years", years.rows_affected());
    let users = sqlx::query!(r#"DELETE FROM "User""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} users", users.rows_affected());
    let tokens = sqlx::query!(r#"DELETE FROM "AccessToken""#)
        .execute(&mut *tx)
        .await?;
    println!("Deleted {} access tokens", tokens.rows_affected());

    tx.commit().await?;

    println!("\nDatabase cleanup completed.");
    println!("Run `cargo run --bin seed` to repopulate the default data.");
    Ok(())
}
