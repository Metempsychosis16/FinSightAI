mod config;
mod models;
mod services;
mod utils;

use axum::{Router, routing::get};

async fn health_check() -> &'static str {
    "Finsigh AI backend is healthy"
}
#[tokio::main]

async fn main() {
    let app = Router::new().route("/health", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Finsigh AI server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
