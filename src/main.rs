use std::sync::Arc;
use tokio::sync::Mutex;

use axum::{Json, Router, extract::{Path, State}, http::{StatusCode, header::LOCATION}, response::{AppendHeaders, IntoResponse}, routing::get};
use serde::{Deserialize, Serialize};
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
    tasks: Vec<Task>,
    next_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

async fn get_tasks(State(app_state): State<Arc<Mutex<AppState>>>) -> Json<Vec<Task>> {
    let app_state_mutex = app_state.lock().await;
    Json(app_state_mutex.tasks.clone())
}

async fn get_task(State(app_state): State<Arc<Mutex<AppState>>>, Path(id): Path<i32>) -> Result<Json<Task>, StatusCode> {
    let app_state_mutex = app_state.lock().await;
    match app_state_mutex.tasks.iter().find(|item| item.id == id) {
        Some(task) => Ok(Json(task.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_task(State(app_state): State<Arc<Mutex<AppState>>>, Json(create_task): Json<CreateTask>) -> Result<impl IntoResponse, (StatusCode, Json<ErrorMessage>)> {
    validate_create_task(&create_task)?;

    let mut app_state_mutex = app_state.lock().await;
    let next_id = app_state_mutex.next_id;
    let task = Task {
        id: next_id,
        title: create_task.title,
        completed: create_task.completed,
    };
    app_state_mutex.tasks.push(task.clone());

    let location = format!("/tasks/{}", next_id);
    let headers = AppendHeaders([(LOCATION,location)]);

    app_state_mutex.next_id += 1;

    Ok((StatusCode::CREATED, headers, Json(task)))
}

async fn update_task(State(app_state): State<Arc<Mutex<AppState>>>, Path(id): Path<i32>, Json(add_task): Json<CreateTask>) -> Result<StatusCode, (StatusCode, Json<ErrorMessage>)> {
    validate_create_task(&add_task)?;

    let mut app_state_mutex = app_state.lock().await;
    match app_state_mutex.tasks.iter_mut().find(|item| item.id == id) {
        Some(task) => {
            task.title = add_task.title;
            task.completed = add_task.completed;
            Ok(StatusCode::OK)
        },
        None => {
            let message = format!("Couldn't find Task with id: {id}");
            Err((StatusCode::NOT_FOUND, Json(ErrorMessage{ error: String::from("task_not_found"), message: message})))
        },
    }
}

fn validate_create_task(task: &CreateTask) -> Result<(), (StatusCode, Json<ErrorMessage>)> {
    if let Err(error) = task.validate() {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorMessage { error: String::from("validation_failed"), message: error.to_string() })));
    }
    Ok(())
}

async fn delete_task(State(app_state): State<Arc<Mutex<AppState>>>, Path(id): Path<i32>) -> Result<StatusCode, StatusCode> {
    let mut app_state_mutex = app_state.lock().await;
    match app_state_mutex.tasks.iter().enumerate().find(|(_,item)| item.id == id) {
        Some((index,_)) => {
            app_state_mutex.tasks.remove(index);
            Ok(StatusCode::NO_CONTENT)
        }
        None => {
            Err(StatusCode::NOT_FOUND)
        },
    }
}


#[tokio::main]
async fn main() {
    let app_state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState{tasks: vec![], next_id: 0}));

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
