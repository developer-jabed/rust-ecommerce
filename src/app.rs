use crate::modules;
use crate::state::AppState;
use axum::{
    routing::get,
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use std::time::Duration;

pub fn create_router(state: AppState) -> Router {
    let cors = if state.config.is_production() {
        CorsLayer::new()
            .allow_origin(
                state
                    .config
                    .frontend_url
                    .parse::<axum::http::HeaderValue>()
                    .expect("Invalid FRONTEND_URL"),
            )
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(true)
    } else {
        CorsLayer::permissive()
    };

    // Public health endpoint (no auth)
    let health = Router::new().route("/health", get(health_check));

    // API v1 routes – modules will register their own sub-routers
    let api_v1 = Router::new()
        .nest("/auth", modules::auth::routes())
        .nest("/users", modules::users::routes())
        .nest("/categories", modules::categories::admin_routes())
        .nest("/brands", modules::brands::routes())
        .nest("/products", modules::products::routes())
        .nest("/search", modules::search::routes())
        .nest("/cart", modules::cart::routes())
        .nest("/checkout", modules::checkout::routes())
        .nest("/orders", modules::orders::routes())
        .nest("/payments", modules::payments::routes())
        .nest("/admin", modules::admin::routes())
        // Placeholder modules (empty routers for now)
        .nest("/inventory", modules::inventory::routes())
        .nest("/promotions", modules::promotions::routes())
        .nest("/shipping", modules::shipping::routes())
        .nest("/reviews", modules::reviews::routes())
        .nest("/analytics", modules::analytics::routes());

    Router::new()
        .merge(health)
        .nest("/api/v1", api_v1)
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors)
        .with_state(state)
}

async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    // Simple readiness: try a cheap DB query
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(state.pool())
        .await
        .is_ok();

    let status = if db_ok {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    let body = serde_json::json!({
        "success": db_ok,
        "status": if db_ok { "healthy" } else { "degraded" },
        "database": if db_ok { "up" } else { "down" },
        "version": env!("CARGO_PKG_VERSION"),
    });

    (status, axum::Json(body))
}