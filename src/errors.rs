use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorMessage {
    error: String,
    message: String,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String, i32),
    Validation(String),
    Database(sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::NotFound(entity, id) => {
                tracing::info!(entity_name = %entity, id = id, "Can't find {} with id {}", entity, id);
                (
                    StatusCode::NOT_FOUND,
                    Json(ErrorMessage {
                        error: String::from("not_found"),
                        message: format!("Couldn't find {entity} with id: {id}"),
                    }),
                )
                    .into_response()
            }
            AppError::Validation(error) => {
                tracing::info!(validation_message = %error, "Validation failed");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorMessage {
                        error: String::from("validation_failed"),
                        message: String::from(error),
                    }),
                )
                    .into_response()
            }
            AppError::Database(error) => {
                tracing::error!(error = %error, "Database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorMessage {
                        error: String::from("internal_server_error"),
                        message: String::from("Something failed"),
                    }),
                )
                    .into_response()
            }
        }
    }
}
