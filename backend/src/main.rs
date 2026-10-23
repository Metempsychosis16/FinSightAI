pub mod auth;
mod config;
mod models;
mod repositories;
mod routes;
mod services;
mod utils;

use crate::config::Config;

use axum::serve;
use sqlx::PgPool;
use tokio::net::TcpListener;

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    tracing::info!("Starting Finsight AI");

    tracing::debug!("Debug logging is enabled");

    let config = Config::from_env().map_err(std::io::Error::other)?;
    let db = PgPool::connect(&config.database_url).await?;

    let state = routes::AppState {
        app_name: String::from("Finsight AI"),
        environment: config.app_env.clone(),
        db,
        jwt_secret: config.jwt_secret,
    };
    let app = routes::create_router(state);

    let address = format!("0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(&address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
