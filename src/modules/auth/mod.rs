//! Authentication module: register, login, refresh, logout, me.
pub mod dto;
mod handlers;
pub mod service;
mod repository;

use axum::{
    routing::{get, post},
    Router,
};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
}
