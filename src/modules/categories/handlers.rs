use super::dto::*;
use super::service::CategoryService;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::middleware::authorization::require_admin;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

pub async fn list_root(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, AppError> {
    let svc = CategoryService::new(state.pool());
    let data = svc.list_root().await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn tree(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, AppError> {
    let svc = CategoryService::new(state.pool());
    let data = svc.tree().await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id_or_slug): Path<String>,
) -> Result<Json<ApiResponse<CategoryResponse>>, AppError> {
    let svc = CategoryService::new(state.pool());
    let data = svc.get_one(&id_or_slug).await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn children(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<CategoryResponse>>>, AppError> {
    let svc = CategoryService::new(state.pool());
    let data = svc.children(id).await?;
    Ok(Json(ApiResponse::new(data)))
}

pub async fn create(
    State(state): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<ApiResponse<CategoryResponse>>), AppError> {
    require_admin(&user)?;
    let svc = CategoryService::new(state.pool());
    let data = svc.create(body).await?;
    Ok((StatusCode::CREATED, Json(ApiResponse::new(data))))
}