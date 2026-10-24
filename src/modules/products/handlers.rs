use super::dto::*;
use super::service::ProductService;
use crate::errors::AppError;
use crate::response::ApiResponse;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde_json::json;

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ProductQuery>,
) -> Result<Json<ApiResponse<Vec<ProductListItem>>>, AppError> {
    let svc = ProductService::new(state.pool());
    let (items, meta) = svc.list(query).await?;
    Ok(Json(ApiResponse::with_meta(
        items,
        json!({ "pagination": meta }),
    )))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id_or_slug): Path<String>,
) -> Result<Json<ApiResponse<ProductDetail>>, AppError> {
    let svc = ProductService::new(state.pool());
    let data = svc.get_one(&id_or_slug).await?;
    Ok(Json(ApiResponse::new(data)))
}
