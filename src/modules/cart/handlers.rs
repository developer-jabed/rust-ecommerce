use super::dto::*;
use super::service::CartService;
use crate::errors::AppError;
use crate::middleware::auth::OptionalAuthUser;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use uuid::Uuid;

fn guest_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-guest-token")
        .and_then(|v| v.to_str().ok())
}

pub async fn get_cart(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<CartResponse>>, AppError> {
    let svc = CartService::new(state.pool());
    let data = svc
        .get_cart(user.as_ref().map(|u| u.user_id), guest_token(&headers))
        .await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn add_item(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    headers: HeaderMap,
    Json(body): Json<AddItemRequest>,
) -> Result<Json<ApiResponse<CartResponse>>, AppError> {
    let svc = CartService::new(state.pool());
    let data = svc
        .add_item(user.as_ref().map(|u| u.user_id), guest_token(&headers), body)
        .await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn update_item(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
    Json(body): Json<UpdateItemRequest>,
) -> Result<Json<ApiResponse<CartResponse>>, AppError> {
    let svc = CartService::new(state.pool());
    let data = svc
        .update_item(user.as_ref().map(|u| u.user_id), guest_token(&headers), item_id, body)
        .await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn remove_item(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    headers: HeaderMap,
    Path(item_id): Path<Uuid>,
) -> Result<Json<ApiResponse<CartResponse>>, AppError> {
    let svc = CartService::new(state.pool());
    let data = svc
        .remove_item(user.as_ref().map(|u| u.user_id), guest_token(&headers), item_id)
        .await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn clear_cart(
    State(state): State<AppState>,
    OptionalAuthUser(user): OptionalAuthUser,
    headers: HeaderMap,
) -> Result<axum::http::StatusCode, AppError> {
    let svc = CartService::new(state.pool());
    svc.clear(user.as_ref().map(|u| u.user_id), guest_token(&headers))
        .await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
