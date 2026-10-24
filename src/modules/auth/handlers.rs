use super::dto::*;
use super::service::AuthService;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

fn client_meta(headers: &HeaderMap) -> (Option<&str>, Option<&str>) {
    let ua = headers.get("user-agent").and_then(|v| v.to_str().ok());

    let ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim());

    (ua, ip)
}

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<ApiResponse<AuthResponse>>), AppError> {
    let svc = AuthService::new(state.pool(), &state.config);
    let resp = svc.register(body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(resp))))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Json<ApiResponse<AuthResponse>>, AppError> {
    let (ua, ip) = client_meta(&headers);
    let svc = AuthService::new(state.pool(), &state.config);
    let resp = svc.login(body, ua, ip).await?;
    Ok(Json(ApiResponse::new(resp)))
}

/// POST /api/v1/auth/refresh
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<ApiResponse<AuthTokens>>, AppError> {
    if body.refresh_token.trim().is_empty() {
        return Err(AppError::unauthorized(
            "Refresh token is required. Please login again.",
        ));
    }

    let (ua, ip) = client_meta(&headers);
    let svc = AuthService::new(state.pool(), &state.config);
    let tokens = svc.refresh(&body.refresh_token, ua, ip).await?;
    Ok(Json(ApiResponse::new(tokens)))
}

/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> Result<StatusCode, AppError> {
    if body.refresh_token.trim().is_empty() {
        return Err(AppError::bad_request("Refresh token is required"));
    }

    let svc = AuthService::new(state.pool(), &state.config);
    svc.logout(&body.refresh_token).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ApiResponse<UserResponse>>, AppError> {
    let svc = AuthService::new(state.pool(), &state.config);
    let resp = svc.me(user.user_id).await?;
    Ok(Json(ApiResponse::new(resp)))
}