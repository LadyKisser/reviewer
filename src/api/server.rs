use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use crate::app::utils::{database::Database, cache::Cache};
use crate::api::routes::AppState;

pub async fn create_server(db: Database, cache: Cache) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState::new(db, cache);
    
    crate::api::routes::create_router()
        .with_state(state)
        .layer(cors)
}