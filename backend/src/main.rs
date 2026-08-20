mod config;
mod models;
mod services;
mod utils;

use axum::{
    Json, Router,
    routing::{get, post},
};

use serde::{Deserialize, Serialize};

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

#[derive(Deserialize)]

struct CreateCaseRequest {
    title: String,
    risk_score: u8,
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

async fn create_case(Json(payload): Json<CreateCaseRequest>) -> Json<HealthResponse> {
    println!("Create Case : {}", payload.title);
    println!("Risk Score : {}", payload.risk_score);

    Json(HealthResponse {
        status: "case recieved",
        service: "Finsight AI",
    })
}

#[tokio::main]

async fn main() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/about", get(about))
        .route("/cases", post(create_case));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Finsigh AI server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
