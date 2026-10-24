//! Authentication middleware – JWT access token verification.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use crate::errors::AppError;
use crate::modules::auth::service::{AccessClaims, AuthService};
use crate::state::AppState;
use uuid::Uuid;

/// Authenticated user context extracted from a valid access token.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let auth_header = match auth_header {
            Some(h) if !h.trim().is_empty() => h.trim(),
            _ => {
                return Err(AppError::unauthorized(
                    "Please log in. Missing Authorization header.",
                ));
            }
        };

        let token = match auth_header.strip_prefix("Bearer ") {
            Some(t) if !t.trim().is_empty() => t.trim(),
            Some(_) => {
                return Err(AppError::unauthorized(
                    "Invalid token. Authorization header is empty after 'Bearer '.",
                ));
            }
            None => {
                return Err(AppError::unauthorized(
                    "Invalid Authorization format. Use: Bearer <your-access-token>",
                ));
            }
        };

        let svc = AuthService::new(state.pool(), &state.config);

        let claims: AccessClaims = svc.decode_access_token(token).map_err(|_| {
            AppError::unauthorized(
                "Session expired or token is invalid. Please log in again.",
            )
        })?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            AppError::unauthorized("Invalid token payload. Please log in again.")
        })?;

        Ok(AuthUser {
            user_id,
            email: claims.email,
            role: claims.role,
        })
    }
}

/// Optional authentication – returns None when no valid token is present.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<AuthUser>);

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(OptionalAuthUser(Some(user))),
            Err(AppError::Unauthorized(_)) => Ok(OptionalAuthUser(None)),
            Err(e) => Err(e),
        }
    }
}