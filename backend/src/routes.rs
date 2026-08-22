use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
}

#[derive(Serialize)]

struct HealthResponse {
    status: &'static str,
    service: String,
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
#[derive(Serialize)]

struct CaseResponse {
    title: String,
    risk_score: u8,
}
struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
}

async fn about() -> Json<AboutResponse> {
    Json(AboutResponse {
        name: "Finsight AI",
        purpose: "AI Financial Crime Investigation Platform",
    })
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "Healthy",
        service: state.app_name.clone(),
    })
}

async fn create_case(
    Json(payload): Json<CreateCaseRequest>,
) -> Result<(StatusCode, Json<CaseResponse>), ApiError> {
    if payload.title.trim().is_empty() {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("title cannot be empty"),
        });
    }

    if payload.title.trim().len() < 5 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("title must be at least 5 characters"),
        });
    }
    if payload.risk_score > 100 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("risk_score must be between 0 to 100"),
        });
    }

    Ok((
        StatusCode::CREATED,
        Json(CaseResponse {
            title: payload.title,
            risk_score: payload.risk_score,
        }),
    ))
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(serde_json::json!({
            "error":self.error,
            "message":self.message,
        }));
        (self.status, body).into_response()
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/cases", post(create_case))
        .route("/about", get(about))
        .with_state(state)
}
