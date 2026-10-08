use crate::models::case_response::{CaseResponse, CaseResponseError, CaseRow};
use crate::models::status::CaseStatus;

use crate::models::validation::Validate;

use crate::repositories::error::RepositoryError;

use crate::services::case_service::{CaseService, CaseServiceError};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub environment: String,
    pub db: PgPool,
}

#[derive(Deserialize)]
struct UpdateCaseStatusRequest {
    status: CaseStatus,
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

impl Validate for CreateCaseRequest {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err(String::from("case title cannot be empty"));
        }

        if !(0..=100).contains(&self.risk_score) {
            return Err(String::from("risk score must be between 0 and 100"));
        }

        Ok(())
    }
}

#[derive(Deserialize)]

struct UpdateCaseRequest {
    title: Option<String>,
    risk_score: Option<u8>,
}

impl Validate for UpdateCaseRequest {
    fn validate(&self) -> Result<(), String> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err(String::from("case title cannot be empty"));
            }
        }
        if let Some(risk_score) = self.risk_score {
            if risk_score > 100 {
                return Err(String::from("risk score must be between 0 and 100"));
            }
        }
        Ok(())
    }
}

struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
}

fn map_case_service_error(error: CaseServiceError) -> ApiError {
    match error {
        CaseServiceError::Repository(RepositoryError::Database(error)) => {
            eprintln!("Database error in case service: {error}");

            ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: "database_error",
                message: String::from("failed to update case status"),
            }
        }

        CaseServiceError::NotFound => ApiError {
            status: StatusCode::NOT_FOUND,
            error: "case_not_found",
            message: String::from("case not found"),
        },

        CaseServiceError::InvalidStoredStatus => ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "invalid_case_status",
            message: String::from("invalid case status in database"),
        },

        CaseServiceError::InvalidTransition => ApiError {
            status: StatusCode::CONFLICT,
            error: "invalid_status_transition",
            message: String::from("case cannot transition to the requested status"),
        },
    }
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
    State(state): State<AppState>,
    Json(payload): Json<CreateCaseRequest>,
) -> Result<(StatusCode, Json<CaseResponse>), ApiError> {
    payload.validate().map_err(|message| ApiError {
        status: StatusCode::BAD_REQUEST,
        error: "validation_error",
        message,
    })?;

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

    let row = sqlx::query_as::<_, CaseRow>(
        r#"
        INSERT INTO cases(title,risk_score)
        VALUES($1,$2)
        RETURNING id,title,risk_score,status,created_at,updated_at
        "#,
    )
    .bind(payload.title.trim())
    .bind(i16::from(payload.risk_score))
    .fetch_one(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while creating case:{error}");
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from("failed to create case"),
        }
    })?;

    let case = CaseResponse::try_from(row)?;

    Ok((StatusCode::CREATED, Json(case)))
}

async fn list_cases(State(state): State<AppState>) -> Result<Json<Vec<CaseResponse>>, ApiError> {
    let cases = sqlx::query_as::<_, CaseRow>(
        r#"
        SELECT id, title, risk_score, status,created_at,updated_at
        FROM cases 
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while listing cases:{error}");

        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from("failed to retrieve cases"),
        }
    })?;

    let cases = cases
        .into_iter()
        .map(CaseResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(cases))
}
async fn list_high_risk_cases(
    State(state): State<AppState>,
) -> Result<Json<Vec<CaseResponse>>, ApiError> {
    let cases = sqlx::query_as::<_, CaseRow>(
        r#"
        SELECT id,title,risk_score,status,created_at,updated_at
        FROM cases
        WHERE risk_score>= 80
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while listing the high risk cases :{error}");

        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from(" Failed to retrieve high risk cases"),
        }
    })?;

    let cases = cases
        .into_iter()
        .map(CaseResponse::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(cases))
}

async fn get_case(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<CaseResponse>, ApiError> {
    let case = sqlx::query_as::<_, CaseRow>(
        r#"
        SELECT id, title, risk_score, status,created_at,updated_at
        FROM cases
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while retrieving case: {error}");

        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from("failed to retrieve case"),
        }
    })?;

    match case {
        Some(row) => Ok(Json(CaseResponse::try_from(row)?)),

        None => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            error: "case_not_found",
            message: String::from("case not found"),
        }),
    }
}

async fn update_case(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCaseRequest>,
) -> Result<Json<CaseResponse>, ApiError> {
    if id <= 0 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("case is must be greater than 0"),
        });
    }

    payload.validate().map_err(|message| ApiError {
        status: StatusCode::BAD_REQUEST,
        error: "validation error",
        message,
    })?;

    if let Some(title) = &payload.title {
        if title.trim().is_empty() {
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                error: "validation_error",
                message: String::from("title cannot be empty"),
            });
        }

        if title.trim().len() < 5 {
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                error: "validation_error",
                message: String::from("title must be at least 5 charachter"),
            });
        }
    }

    if let Some(risk_score) = payload.risk_score {
        if risk_score > 100 {
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                error: "validation_error",
                message: String::from("risk score must be between 0 and 100"),
            });
        }
    }

    let existing = sqlx::query_as::<_, CaseRow>(
        r#"
    SELECT id,title,risk_score,created_at,status,updated_at
    FROM cases
    WHERE id=$1
    "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while finding case: {error}");

        ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation error",
            message: String::from("failed to retrive case"),
        }
    })?;

    let existing = match existing {
        Some(case) => case,
        None => {
            return Err(ApiError {
                status: StatusCode::NOT_FOUND,
                error: "case_not_found",
                message: String::from("case not found"),
            });
        }
    };

    let title = payload.title.unwrap_or(existing.title);

    let risk_score = payload
        .risk_score
        .map(i16::from)
        .unwrap_or(existing.risk_score);

    let row = sqlx::query_as::<_, CaseRow>(
        r#"
        UPDATE cases
        SET title=$1,
        risk_score=$2,
        updated_at=NOW()
        WHERE id=$3
        RETURNING id,title,risk_score,status,created_at,updated_at
        "#,
    )
    .bind(title.trim())
    .bind(risk_score)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while updating case: {error}");

        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from("failed to update case"),
        }
    })?;

    let updated_case = CaseResponse::try_from(row)?;

    Ok(Json(updated_case))
}

async fn delete_case(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if id <= 0 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("case id must be greater than 0"),
        });
    }

    let result = sqlx::query(
        r#"
        DELETE FROM cases
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|error| {
        eprintln!("Database error while deleting case: {error}");

        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: String::from("failed to delete case"),
        }
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError {
            status: StatusCode::NOT_FOUND,
            error: "case_not_found",
            message: String::from("case not found"),
        });
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn database_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|error| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "database_error",
            message: error.to_string(),
        })?;
    Ok(Json(serde_json::json!({

        "status":"health",
        "database":"connected"
    })))
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

async fn update_case_status(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateCaseStatusRequest>,
) -> Result<Json<CaseResponse>, ApiError> {
    if id <= 0 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("case id must be greater than 0"),
        });
    }

    let service = CaseService::new(&state.db);

    let row = service
        .update_status(id, payload.status)
        .await
        .map_err(map_case_service_error)?;

    Ok(Json(CaseResponse::try_from(row)?))
}

impl From<CaseResponseError> for ApiError {
    fn from(error: CaseResponseError) -> Self {
        match error {
            CaseResponseError::InvalidStatus => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: "invalid_case_status",
                message: String::from("invalid case status in database"),
            },
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/cases", get(list_cases).post(create_case))
        .route("/cases/high-risk", get(list_high_risk_cases))
        .route(
            "/cases/{id}",
            get(get_case).patch(update_case).delete(delete_case),
        )
        .route("/cases/{id}/status", patch(update_case_status))
        .route("/about", get(about))
        .route("/health/database", get(database_health))
        .with_state(state)
}
