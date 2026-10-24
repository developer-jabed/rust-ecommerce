use crate::config::Config;
use crate::infrastructure::database::Database;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Database,
}

impl AppState {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let db = Database::connect(&config).await?;
        Ok(Self {
            config: Arc::new(config),
            db,
        })
    }

    pub fn pool(&self) -> &PgPool {
        &self.db.pool
    }
}