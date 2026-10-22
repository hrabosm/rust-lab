use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header::LOCATION},
    response::{AppendHeaders, IntoResponse},
};
use validator::Validate;

use crate::{dtos::task::CreateTask, errors::AppError, models::task::Task, state::AppState};

fn validate_create_task(task: &CreateTask) -> Result<(), AppError> {
    if let Err(error) = task.validate() {
        return Err(AppError::Validation(error.to_string()));
    }
    Ok(())
}

pub async fn get_tasks(
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let query = sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks;");
    let tasks = query
        .fetch_all(&app_state.pool)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(tasks))
}

pub async fn get_task(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let query =
        sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks WHERE id = $1;").bind(id);

    let task_result = query.fetch_optional(&app_state.pool).await;

    match task_result {
        Ok(Some(task)) => Ok(Json(task)),
        Ok(None) => Err(AppError::NotFound(String::from("Task"), id)),
        Err(error) => Err(AppError::Database(error)),
    }
}

pub async fn create_task(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateTask>,
) -> Result<impl IntoResponse, AppError> {
    validate_create_task(&payload)?;

    let query = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, completed) VALUES ($1, $2) RETURNING id, title, completed;",
    )
    .bind(&payload.title)
    .bind(&payload.completed);

    let task = query
        .fetch_one(&app_state.pool)
        .await
        .map_err(AppError::Database)?;

    let location = format!("/tasks/{}", task.id);
    let headers = AppendHeaders([(LOCATION, location)]);

    Ok((StatusCode::CREATED, headers, Json(task)))
}

pub async fn update_task(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<CreateTask>,
) -> Result<impl IntoResponse, AppError> {
    validate_create_task(&payload)?;

    let query = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, completed = $2 WHERE id = $3 RETURNING id, title, completed;",
    )
    .bind(&payload.title)
    .bind(&payload.completed)
    .bind(&id);

    let task = query
        .fetch_optional(&app_state.pool)
        .await
        .map_err(AppError::Database)?;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(AppError::NotFound(String::from("Task"), id)),
    }
}

pub async fn delete_task(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1;")
        .bind(&id)
        .execute(&app_state.pool)
        .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 1 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err(AppError::NotFound(String::from("Task"), id))
            }
        }
        Err(error) => Err(AppError::Database(error)),
    }
}
