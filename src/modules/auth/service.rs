use super::dto::*;
use super::repository::{self, UserRow};
use crate::config::Config;
use crate::errors::{validation_errors, AppError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String, // user id
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

pub struct AuthService<'a> {
    pool: &'a PgPool,
    config: &'a Config,
}

impl<'a> AuthService<'a> {
    pub fn new(pool: &'a PgPool, config: &'a Config) -> Self {
        Self { pool, config }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        req.validate().map_err(validation_errors)?;

        let email = req.email.trim().to_lowercase();
        if email.is_empty() {
            return Err(AppError::validation("Email is required"));
        }

        if repository::find_by_email(self.pool, &email).await?.is_some() {
            return Err(AppError::conflict(
                "This email is already registered. Please login instead.",
            ));
        }

        let password_hash = hash_password(&req.password)?;
        let user = repository::create_user(
            self.pool,
            &email,
            &password_hash,
            req.full_name.as_deref(),
            req.phone.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create user");
            AppError::Internal
        })?;

        let tokens = self.issue_tokens(&user, None, None).await?;
        Ok(AuthResponse {
            user: user_to_response(&user),
            tokens,
        })
    }

    pub async fn login(
        &self,
        req: LoginRequest,
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<AuthResponse, AppError> {
        req.validate().map_err(validation_errors)?;

        let email = req.email.trim().to_lowercase();
        if email.is_empty() || req.password.is_empty() {
            return Err(AppError::unauthorized("Email and password are required"));
        }

        let user = repository::find_by_email(self.pool, &email)
            .await?
            .ok_or_else(|| AppError::unauthorized("Invalid email or password"))?;

        if !user.is_active {
            return Err(AppError::forbidden(
                "Your account has been disabled. Please contact support.",
            ));
        }

        if !verify_password(&req.password, &user.password_hash)? {
            return Err(AppError::unauthorized("Invalid email or password"));
        }

        let tokens = self.issue_tokens(&user, user_agent, ip).await?;
        Ok(AuthResponse {
            user: user_to_response(&user),
            tokens,
        })
    }

    pub async fn refresh(
        &self,
        refresh_token: &str,
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<AuthTokens, AppError> {
        let refresh_token = refresh_token.trim();
        if refresh_token.is_empty() {
            return Err(AppError::unauthorized(
                "Refresh token is required. Please login again.",
            ));
        }

        let token_hash = hash_token(refresh_token);
        let (user_id, expires_at, revoked_at) =
            repository::find_refresh_token(self.pool, &token_hash)
                .await?
                .ok_or_else(|| {
                    AppError::unauthorized("Invalid refresh token. Please login again.")
                })?;

        if revoked_at.is_some() {
            // Possible reuse attack → revoke all sessions for this user
            let _ = repository::revoke_all_user_tokens(self.pool, user_id).await;
            return Err(AppError::unauthorized(
                "This session is no longer valid. Please login again.",
            ));
        }

        if expires_at < Utc::now() {
            return Err(AppError::unauthorized(
                "Your session has expired. Please login again.",
            ));
        }

        // Rotate: revoke old token, issue new pair
        repository::revoke_refresh_token(self.pool, &token_hash).await?;

        let user = repository::find_by_id(self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::unauthorized("User not found. Please login again."))?;

        if !user.is_active {
            return Err(AppError::forbidden(
                "Your account has been disabled. Please contact support.",
            ));
        }

        self.issue_tokens(&user, user_agent, ip).await
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        let refresh_token = refresh_token.trim();
        if refresh_token.is_empty() {
            return Err(AppError::bad_request("Refresh token is required"));
        }

        let token_hash = hash_token(refresh_token);
        repository::revoke_refresh_token(self.pool, &token_hash).await?;
        Ok(())
    }

    pub async fn me(&self, user_id: Uuid) -> Result<UserResponse, AppError> {
        let user = repository::find_by_id(self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?;

        if !user.is_active {
            return Err(AppError::forbidden(
                "Your account has been disabled. Please contact support.",
            ));
        }

        Ok(user_to_response(&user))
    }

    async fn issue_tokens(
        &self,
        user: &UserRow,
        user_agent: Option<&str>,
        ip: Option<&str>,
    ) -> Result<AuthTokens, AppError> {
        let now = Utc::now();
        let access_exp = now + Duration::seconds(self.config.jwt_access_expiry_secs);
        let jti = Uuid::new_v4().to_string();

        let claims = AccessClaims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            exp: access_exp.timestamp(),
            iat: now.timestamp(),
            jti,
        };

        let access_token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_access_secret.as_bytes()),
        )
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to encode access token");
            AppError::Internal
        })?;

        // Opaque refresh token (only hash stored in DB)
        let refresh_raw = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let refresh_hash = hash_token(&refresh_raw);
        let refresh_exp = now + Duration::seconds(self.config.jwt_refresh_expiry_secs);

        repository::store_refresh_token(
            self.pool,
            user.id,
            &refresh_hash,
            refresh_exp,
            user_agent,
            ip,
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to store refresh token");
            AppError::Internal
        })?;

        Ok(AuthTokens {
            access_token,
            refresh_token: refresh_raw,
            token_type: "Bearer".into(),
            expires_in: self.config.jwt_access_expiry_secs,
        })
    }

    pub fn decode_access_token(&self, token: &str) -> Result<AccessClaims, AppError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(AppError::unauthorized(
                "Access token is missing. Please login.",
            ));
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        decode::<AccessClaims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_access_secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| {
            AppError::unauthorized("Session expired or token is invalid. Please login again.")
        })
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            tracing::error!(error = %e, "Password hashing failed");
            AppError::Internal
        })
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(hash).map_err(|e| {
        tracing::error!(error = %e, "Invalid password hash in database");
        AppError::Internal
    })?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn user_to_response(u: &UserRow) -> UserResponse {
    UserResponse {
        id: u.id,
        email: u.email.clone(),
        full_name: u.full_name.clone(),
        phone: u.phone.clone(),
        role: u.role.clone(),
        email_verified: u.email_verified,
        created_at: u.created_at,
    }
}
