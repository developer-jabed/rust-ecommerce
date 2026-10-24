//! promotions module – full implementation arrives in later phases.
use axum::Router;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
