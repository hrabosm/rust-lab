use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    message: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: String::from("ok"),
        message: String::from("API is running!"),
    })
}
