mod handlers;
mod repository;
mod service;
mod dto;

use axum::{routing::get, Router};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::list))
        .route("/:id_or_slug", get(handlers::get_one))
}
