mod config;
mod models;
mod routes;
mod services;
mod utils;

use axum::serve;
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]

async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreqSql");

    let state = routes::AppState {
        app_name: String::from("Finsight AI"),
        environment: String::from("development"),
        db,
    };
    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Finsigh AI server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
