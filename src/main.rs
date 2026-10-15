use std::sync::Arc;

use axum::{Json, Router, extract::{Path, State}, http::{StatusCode, header::LOCATION}, response::{AppendHeaders, IntoResponse}, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use validator::{Validate, ValidationError};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorMessage {
    error: String,
    message: String,
}

struct AppState {
    pool: sqlx::PgPool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
struct Task {
    id: i32,
    title: String,
    completed: bool,
}

#[derive(Deserialize, Validate)]
struct CreateTask {
    #[validate(length(min = 1), custom(function = "validate_title"))]
    title: String,
    completed: bool,
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().is_empty() {
        return Err(ValidationError::new("whitespace_in_title"))
    }

    Ok(())
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: String::from("ok"),
        message: String::from("API is running!"),
    })
}

async fn get_tasks(State(app_state): State<Arc<AppState>>) -> Result<Json<Vec<Task>>, StatusCode> {
    let query = sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks;");
    let tasks = query.fetch_all(&app_state.pool).await;

    match tasks {
        Ok(tasks) => Ok(Json(tasks)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_task(State(app_state): State<Arc<AppState>>, Path(id): Path<i32>) -> Result<Json<Task>, StatusCode> {
    let query = sqlx::query_as::<_, Task>("SELECT id, title, completed FROM tasks WHERE id = $1;")
        .bind(id);

    let task_result = query.fetch_optional(&app_state.pool).await;

    match task_result {
        Ok(Some(task)) => Ok(Json(task)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_task(State(app_state): State<Arc<AppState>>, Json(create_task): Json<CreateTask>) -> Result<impl IntoResponse, (StatusCode, Json<ErrorMessage>)> {
    validate_create_task(&create_task)?;

    let query = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks (title, completed) VALUES ($1, $2) RETURNING id, title, completed;"
    ).bind(&create_task.title).bind(&create_task.completed);

    let task_result = query.fetch_one(&app_state.pool).await;

    match task_result {
        Ok(task) => {
            let location = format!("/tasks/{}", task.id);
            let headers = AppendHeaders([(LOCATION,location)]);

            Ok((StatusCode::CREATED, headers, Json(task)))
        }
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage{error: String::from("internal_server_error"), message: String::from("Insert failed")}))),
    }
}

async fn update_task(State(app_state): State<Arc<AppState>>, Path(id): Path<i32>, Json(add_task): Json<CreateTask>) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    validate_create_task(&add_task)?;

    let query = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET title = $1, completed = $2 WHERE id = $3 RETURNING id, title, completed;"
    ).bind(&add_task.title).bind(&add_task.completed).bind(&id);

    let task_result = query.fetch_optional(&app_state.pool).await;

    match task_result {
        Ok(Some(_)) => Ok(StatusCode::OK),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(ErrorMessage{ error: String::from("not_found"), message: format!("Couldn't find Task with id: {id}")}))),
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorMessage{error: String::from("internal_server_error"), message: format!("Update failed")}))),
    }
}

fn validate_create_task(task: &CreateTask) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    if let Err(error) = task.validate() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { error: String::from("validation_failed"), message: error.to_string() })));
    }
    Ok(())
}

async fn delete_task(State(app_state): State<Arc<AppState>>, Path(id): Path<i32>) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM tasks WHERE id = $1;"
    ).bind(&id).execute(&app_state.pool).await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 1 {
                Ok(StatusCode::NO_CONTENT)
            }
            else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => {
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    println!("Connected to PostgreSQL!");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    println!("Database migrations applied!");

    let app_state = Arc::new(AppState{pool});

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task).put(update_task).delete(delete_task))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to address");

    println!("Listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}