mod handlers;
mod repository;
mod service;
mod dto;

use axum::{routing::{get, post, patch}, Router};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::get_cart).delete(handlers::clear_cart))
        .route("/items", post(handlers::add_item))
        .route("/items/:item_id", patch(handlers::update_item).delete(handlers::remove_item))
}
