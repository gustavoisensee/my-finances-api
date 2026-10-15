//! Seeds the default years and the global expense categories. Safe to re-run.

use std::env;

use my_finances_api::state::BoxError;
use my_finances_api::time;
use sqlx::postgres::PgPoolOptions;

const DEFAULT_YEARS: [i32; 3] = [2024, 2025, 2026];

const DEFAULT_CATEGORIES: [&str; 11] = [
    "Housing",
    "Transportation",
    "Food & Dining",
    "Utilities",
    "Healthcare",
    "Entertainment",
    "Shopping",
    "Personal Care",
    "Education",
    "Savings & Investments",
    "Other",
];

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    dotenvy::dotenv().ok();
    let url = env::var("DATABASE_URL").map_err(|_| "DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;
    let mut tx = pool.begin().await?;

    println!("Starting database seeding...\n");

    println!("Creating default years...");
    for year in DEFAULT_YEARS {
        sqlx::query!(
            r#"
            INSERT INTO "Year" (value, "createdAt")
            SELECT $1, $2 WHERE NOT EXISTS (SELECT 1 FROM "Year" WHERE value = $1)
            "#,
            year,
            time::now(),
        )
        .execute(&mut *tx)
        .await?;
    }

    println!("Creating default categories...");
    for name in DEFAULT_CATEGORIES {
        sqlx::query!(
            r#"
            INSERT INTO "Category" (name, "userId", "createdAt")
            SELECT $1, NULL, $2
            WHERE NOT EXISTS (SELECT 1 FROM "Category" WHERE name = $1 AND "userId" IS NULL)
            "#,
            name,
            time::now(),
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    println!("\nDatabase seeding completed.");
    println!(
        "  - {} years ({}-{})",
        DEFAULT_YEARS.len(),
        DEFAULT_YEARS[0],
        DEFAULT_YEARS[DEFAULT_YEARS.len() - 1]
    );
    println!("  - {} default categories", DEFAULT_CATEGORIES.len());
    println!("Users are created on their first Firebase sign-in.");
    Ok(())
}
