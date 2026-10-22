use std::sync::Arc;

use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;

use crate::{
    handlers::{
        health::health_check,
        tasks::{create_task, delete_task, get_task, get_tasks, update_task},
    },
    state::AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/tasks", get(get_tasks).post(create_task))
        .route(
            "/tasks/{id}",
            get(get_task).put(update_task).delete(delete_task),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state)
}
