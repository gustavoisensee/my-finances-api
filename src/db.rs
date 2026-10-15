use std::env;
use std::str::FromStr;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

pub const DEFAULT_TAKE: i64 = 10;
pub const MAX_TAKE: i64 = 100;

const DEFAULT_MAX_CONNECTIONS: u32 = 5;

/// Builds a lazily-connecting pool. Each serverless instance creates it once at
/// startup, so warm invocations reuse the same connections.
///
/// PgBouncer/Supavisor in transaction mode cannot keep prepared statements
/// across transactions, so the statement cache is disabled when the URL carries
/// `pgbouncer=true` (the Prisma convention) or `DB_STATEMENT_CACHE_CAPACITY=0`.
pub fn connect_lazy(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let mut options = PgConnectOptions::from_str(database_url)?;

    let cache_capacity = env::var("DB_STATEMENT_CACHE_CAPACITY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .or_else(|| uses_pgbouncer(database_url).then_some(0));
    if let Some(capacity) = cache_capacity {
        options = options.statement_cache_capacity(capacity);
    }

    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(DEFAULT_MAX_CONNECTIONS);

    Ok(PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(60))
        .connect_lazy_with(options))
}

fn uses_pgbouncer(url: &str) -> bool {
    url.split_once('?').is_some_and(|(_, query)| {
        query
            .split('&')
            .any(|pair| pair.eq_ignore_ascii_case("pgbouncer=true"))
    })
}

#[cfg(test)]
mod tests {
    use super::uses_pgbouncer;

    #[test]
    fn detects_pgbouncer_flag() {
        assert!(uses_pgbouncer("postgres://h:6543/db?pgbouncer=true"));
        assert!(uses_pgbouncer(
            "postgres://h/db?sslmode=require&pgbouncer=true"
        ));
        assert!(!uses_pgbouncer("postgres://h/db?sslmode=require"));
        assert!(!uses_pgbouncer("postgres://h/db"));
    }
}
