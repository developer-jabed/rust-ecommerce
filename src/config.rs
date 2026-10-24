use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub frontend_url: String,
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub jwt_access_expiry_secs: i64,
    pub jwt_refresh_expiry_secs: i64,
    pub cloudinary_cloud_name: Option<String>,
    pub cloudinary_api_key: Option<String>,
    pub cloudinary_api_secret: Option<String>,
    pub cloudinary_upload_preset: Option<String>,
    pub payment_provider: String,
    pub payment_store_id: Option<String>,
    pub payment_store_password: Option<String>,
    pub payment_base_url: Option<String>,
    pub payment_success_url: Option<String>,
    pub payment_fail_url: Option<String>,
    pub payment_cancel_url: Option<String>,
    pub payment_ipn_url: Option<String>,
    pub rate_limit_auth: u32,
    pub rate_limit_general: u32,
    pub db_max_connections: u32,
    pub db_min_connections: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .map_err(|_| anyhow::anyhow!("PORT must be a valid u16"))?,
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".into()),
            database_url: env::var("DATABASE_URL")
                .map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?,
            jwt_access_secret: env::var("JWT_ACCESS_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_ACCESS_SECRET is required"))?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .map_err(|_| anyhow::anyhow!("JWT_REFRESH_SECRET is required"))?,
            jwt_access_expiry_secs: env::var("JWT_ACCESS_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".into())
                .parse()
                .unwrap_or(900),
            jwt_refresh_expiry_secs: env::var("JWT_REFRESH_EXPIRY_SECS")
                .unwrap_or_else(|_| "604800".into())
                .parse()
                .unwrap_or(604800),
            cloudinary_cloud_name: env::var("CLOUDINARY_CLOUD_NAME").ok(),
            cloudinary_api_key: env::var("CLOUDINARY_API_KEY").ok(),
            cloudinary_api_secret: env::var("CLOUDINARY_API_SECRET").ok(),
            cloudinary_upload_preset: env::var("CLOUDINARY_UPLOAD_PRESET").ok(),
            payment_provider: env::var("PAYMENT_PROVIDER")
                .unwrap_or_else(|_| "sslcommerz".into()),
            payment_store_id: env::var("PAYMENT_STORE_ID").ok(),
            payment_store_password: env::var("PAYMENT_STORE_PASSWORD").ok(),
            payment_base_url: env::var("PAYMENT_BASE_URL").ok(),
            payment_success_url: env::var("PAYMENT_SUCCESS_URL").ok(),
            payment_fail_url: env::var("PAYMENT_FAIL_URL").ok(),
            payment_cancel_url: env::var("PAYMENT_CANCEL_URL").ok(),
            payment_ipn_url: env::var("PAYMENT_IPN_URL").ok(),
            rate_limit_auth: env::var("RATE_LIMIT_AUTH")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            rate_limit_general: env::var("RATE_LIMIT_GENERAL")
                .unwrap_or_else(|_| "120".into())
                .parse()
                .unwrap_or(120),
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".into())
                .parse()
                .unwrap_or(10),
            db_min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "2".into())
                .parse()
                .unwrap_or(2),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
    }
}