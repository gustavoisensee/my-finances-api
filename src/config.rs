use std::collections::HashSet;
use std::env;

pub const PRODUCTION_ORIGIN: &str = "https://my-finances-web.gustavoisensee.com";
pub const LOCAL_ORIGIN: &str = "http://localhost:3000";
/// The API docs page served by `make docs`.
pub const DOCS_ORIGIN: &str = "http://localhost:3002";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required environment variable {0}")]
    Missing(&'static str),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub firebase_project_id: String,
    pub admin_uids: HashSet<String>,
    pub is_production: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            firebase_project_id: required("FIREBASE_PROJECT_ID")?,
            admin_uids: parse_admin_uids(env::var("ADMIN_FIREBASE_UIDS").ok().as_deref()),
            is_production: is_production_env(
                env::var("APP_ENV").ok().as_deref(),
                env::var("VERCEL_ENV").ok().as_deref(),
            ),
        })
    }

    pub fn allowed_origins(&self) -> Vec<&'static str> {
        let mut origins = vec![PRODUCTION_ORIGIN];
        if !self.is_production {
            origins.extend([LOCAL_ORIGIN, DOCS_ORIGIN]);
        }
        origins
    }
}

fn required(key: &'static str) -> Result<String, ConfigError> {
    env::var(key)
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .ok_or(ConfigError::Missing(key))
}

/// `APP_ENV` wins; otherwise fall back to Vercel's `VERCEL_ENV`.
pub fn is_production_env(app_env: Option<&str>, vercel_env: Option<&str>) -> bool {
    match app_env.map(str::trim).filter(|v| !v.is_empty()) {
        Some(v) => v.eq_ignore_ascii_case("production"),
        None => vercel_env.is_some_and(|v| v.trim().eq_ignore_ascii_case("production")),
    }
}

pub fn parse_admin_uids(raw: Option<&str>) -> HashSet<String> {
    raw.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_detection() {
        assert!(is_production_env(Some("production"), None));
        assert!(!is_production_env(Some("development"), Some("production")));
        assert!(is_production_env(None, Some("production")));
        assert!(!is_production_env(None, Some("preview")));
        assert!(!is_production_env(None, None));
        assert!(!is_production_env(Some(" "), None));
    }

    #[test]
    fn admin_uids_are_trimmed() {
        let uids = parse_admin_uids(Some(" a, b ,,c "));
        assert_eq!(uids.len(), 3);
        assert!(uids.contains("a") && uids.contains("b") && uids.contains("c"));
        assert!(parse_admin_uids(None).is_empty());
    }
}
