use std::sync::Arc;

use sqlx::PgPool;

use crate::auth::firebase::FirebaseVerifier;
use crate::config::Config;
use crate::db;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: PgPool,
    pub verifier: Arc<FirebaseVerifier>,
}

impl AppState {
    pub fn new(config: Config, db: PgPool, verifier: FirebaseVerifier) -> Self {
        Self {
            config: Arc::new(config),
            db,
            verifier: Arc::new(verifier),
        }
    }

    pub fn from_env() -> Result<Self, BoxError> {
        let config = Config::from_env()?;
        let pool = db::connect_lazy(&config.database_url)?;
        let verifier = FirebaseVerifier::google(&config.firebase_project_id)?;
        Ok(Self::new(config, pool, verifier))
    }
}
