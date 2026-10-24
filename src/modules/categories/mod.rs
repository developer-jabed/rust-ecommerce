mod handlers;
mod repository;
mod service;
mod dto;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list_root))
        .route("/tree", get(handlers::tree))
        .route("/:id_or_slug", get(handlers::get_one))
        .route("/:id/children", get(handlers::children))
}

/// Admin routes – nest under /api/v1/admin/categories when ready
pub fn admin_routes() -> Router<AppState> {
    Router::new().route("/", post(handlers::create))
}