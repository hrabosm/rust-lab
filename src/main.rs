mod config;
mod dtos;
mod errors;
mod handlers;
mod models;
mod routes;
mod state;

use crate::{config::Config, state::AppState};

use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();

    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!(name: "PostgreSQL Conenction", "Connected to PostgreSQL");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!(name: "migrations", "Database migrations finished");

    let app_state = Arc::new(AppState { pool });
    let app = routes::create_router(app_state);

    let listener = tokio::net::TcpListener::bind(&config.server_address)
        .await
        .expect("Failed to bind to address");

    tracing::info!(name: "Ready", "Listening on {}", &config.server_address);

    axum::serve(listener, app).await.expect("Server failed");
}
