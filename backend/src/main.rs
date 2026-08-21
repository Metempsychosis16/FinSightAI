mod config;
mod models;
mod routes;
mod services;
mod utils;

use axum::serve;
use tokio::net::TcpListener;

#[tokio::main]

async fn main() {
    let app = routes::create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Finsigh AI server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
