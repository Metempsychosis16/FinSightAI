mod config;
mod models;
mod services;
mod utils;

use axum::{Json, Router, routing::get};

use serde::Serialize;

#[derive(Serialize)]

struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]

struct AboutResponse {
    name: &'static str,
    purpose: &'static str,
}

async fn about() -> Json<AboutResponse> {
    Json(AboutResponse {
        name: "Finsight AI",
        purpose: "AI Financial Crime Investigation Platform",
    })
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "Healthy",
        service: "Finsight AI",
    })
}
#[tokio::main]

async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/about", get(about));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Finsigh AI server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
