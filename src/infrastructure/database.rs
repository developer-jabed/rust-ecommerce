use crate::config::Config;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(config: &Config) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.db_max_connections)
            .min_connections(config.db_min_connections)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .max_lifetime(Duration::from_secs(1800))
            .connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        // Run migrations automatically on startup in non-production if desired.
        // In production prefer explicit migration commands.
        if !config.is_production() {
            tracing::info!("Running database migrations (development)");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
        }

        Ok(Self { pool })
    }
}