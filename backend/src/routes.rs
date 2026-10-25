use crate::models::case_response::{CaseResponse, CaseResponseError, CaseRow};
use crate::models::status::CaseStatus;

use crate::models::validation::Validate;

use crate::repositories::error::RepositoryError;

use tower_http::trace::TraceLayer;

use crate::models::role::Role;

use crate::services::bulk_transaction_service::{BulkImportError, import_csv};

use crate::services::case_service::{CaseService, CaseServiceError};

use crate::auth::middleware::require_auth;
use axum::middleware::from_fn_with_state;

use crate::models::user_response::UserResponse;
use crate::services::registration_service::{RegistrationError, register_user};

use crate::auth::jwt::create_token;
use crate::services::login_service::{LoginError, authenticate_user};

use crate::services::case_assignment_service::{CaseAssignmentService, CaseAssignmentServiceError};

use std::time::{SystemTime, UNIX_EPOCH};

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

use crate::models::transaction::Transaction;
use crate::services::transaction_service::{TransactionService, TransactionServiceError};

#[derive(Debug, serde::Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub app_name: String,
    pub environment: String,
    pub db: PgPool,
    pub jwt_secret: String,
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

#[derive(Debug, Deserialize)]
struct CreateTransactionRequest {
    external_reference: String,
    amount_minor: i64,
    currency: String,
    sender_account: String,
    recipient_account: String,
    transaction_type: String,
    occurred_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct TransactionResponse {
    id: i64,
    external_reference: String,
    amount_minor: i64,
    currency: String,
    sender_account: String,
    recipient_account: String,
    transaction_type: &'static str,
    occurred_at: DateTime<Utc>,
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        Self {
            id: transaction.id,
            external_reference: transaction.external_reference,
            amount_minor: transaction.amount_minor,
            currency: transaction.currency,
            sender_account: transaction.sender_account,
            recipient_account: transaction.recipient_account,
            transaction_type: transaction.transaction_type.as_str(),
            occurred_at: transaction.occurred_at,
        }
    }
}

#[derive(Debug, Deserialize)]
struct AssignInvestigatorRequest {
    investigator_id: i64,
}

#[derive(Debug, Serialize)]
struct AssignInvestigatorResponse {
    case_id: i64,
    investigator_id: i64,
    message: &'static str,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    match register_user(&state.db, &payload.email, &payload.password).await {
        Ok(user) => {
            let response = UserResponse::from(user);

            (StatusCode::CREATED, Json(response)).into_response()
        }

        Err(RegistrationError::InvalidEmail) | Err(RegistrationError::WeakPassword) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid registration input"
            })),
        )
            .into_response(),

        Err(RegistrationError::DuplicateEmail) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "An account with this email already exists"
            })),
        )
            .into_response(),

        Err(RegistrationError::PasswordHashingFailed) | Err(RegistrationError::Database(_)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Registration could not be completed"
            })),
        )
            .into_response(),
    }
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

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state.db, &payload.email, &payload.password).await {
        Ok(user) => user,

        Err(LoginError::InvalidCredentials) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid email or password"
                })),
            )
                .into_response();
        }

        Err(LoginError::Database(error)) => {
            tracing::error!(
                error = %error,
                "Login database operation failed"
            );

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Login could not be completed"
                })),
            )
                .into_response();
        }
    };

    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as usize,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Login could not be completed"
                })),
            )
                .into_response();
        }
    };

    let expires_in = 900_u64;

    let role = match Role::parse(&user.role) {
        Some(role) => role,
        None => {
            tracing::error!(user_id = user.id, "User has an unsupported role");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Login could not be completed"
                })),
            )
                .into_response();
        }
    };

    let token = match create_token(
        user.id,
        role.as_str(),
        &state.jwt_secret,
        now,
        expires_in as usize,
    ) {
        Ok(token) => token,

        Err(_) => {
            tracing::error!("JWT creation failed");

            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Login could not be completed"
                })),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(LoginResponse {
            access_token: token,
            token_type: "Bearer",
            expires_in,
        }),
    )
        .into_response()
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
            tracing::error!(
                error_category = "database_error",
                error = %error,
                "Case status update failed"
            );

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

async fn assign_investigator(
    State(state): State<AppState>,
    Path(case_id): Path<i64>,
    Json(payload): Json<AssignInvestigatorRequest>,
) -> Result<(StatusCode, Json<AssignInvestigatorResponse>), ApiError> {
    if case_id <= 0 || payload.investigator_id <= 0 {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message: String::from("case_id and investigator_id must be greater than zero"),
        });
    }

    let service = CaseAssignmentService::new(&state.db);

    match service.assign(case_id, payload.investigator_id).await {
        Ok(()) => Ok((
            StatusCode::OK,
            Json(AssignInvestigatorResponse {
                case_id,
                investigator_id: payload.investigator_id,
                message: "Investigator assigned successfully",
            }),
        )),

        Err(CaseAssignmentServiceError::CaseNotFound) => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            error: "case_not_found",
            message: String::from("Case not found"),
        }),

        Err(CaseAssignmentServiceError::InvestigatorNotAssignable) => Err(ApiError {
            status: StatusCode::NOT_FOUND,
            error: "investigator_not_assignable",
            message: String::from("Investigator not found or account is inactive"),
        }),

        Err(CaseAssignmentServiceError::Database(error)) => {
            tracing::error!(
                error = %error,
                case_id,
                investigator_id = payload.investigator_id,
                "Case investigator assignment failed"
            );

            Err(ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: "database_error",
                message: String::from("Failed to assign investigator"),
            })
        }
    }
}

async fn import_transactions_csv(
    State(state): State<AppState>,
    body: String,
) -> Result<
    (
        StatusCode,
        Json<crate::services::bulk_transaction_service::BulkImportReport>,
    ),
    ApiError,
> {
    match import_csv(&state.db, &body).await {
        Ok(report) => Ok((StatusCode::OK, Json(report))),

        Err(BulkImportError::FileTooLarge) => Err(ApiError {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            error: "csv_too_large",
            message: String::from("CSV file exceeds the 1 MB limit"),
        }),

        Err(BulkImportError::TooManyRows) => Err(ApiError {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            error: "too_many_rows",
            message: String::from("CSV file exceeds the 500-row limit"),
        }),

        Err(BulkImportError::MissingHeaders(headers)) => Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "missing_csv_headers",
            message: format!("Missing required CSV headers: {}", headers.join(", ")),
        }),

        Err(BulkImportError::InvalidCsv(_)) => Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "invalid_csv",
            message: String::from("CSV file or header row is invalid"),
        }),
    }
}

async fn create_transaction(
    State(state): State<AppState>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), ApiError> {
    let service = TransactionService::new(&state.db);

    match service
        .ingest(
            payload.external_reference,
            payload.amount_minor,
            payload.currency,
            payload.sender_account,
            payload.recipient_account,
            &payload.transaction_type,
            payload.occurred_at,
        )
        .await
    {
        Ok(transaction) => Ok((
            StatusCode::CREATED,
            Json(TransactionResponse::from(transaction)),
        )),

        Err(TransactionServiceError::InvalidInput(message)) => Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            error: "validation_error",
            message,
        }),

        Err(TransactionServiceError::DuplicateReference) => Err(ApiError {
            status: StatusCode::CONFLICT,
            error: "duplicate_transaction",
            message: String::from("A transaction with this external reference already exists"),
        }),

        Err(TransactionServiceError::Repository(error)) => {
            tracing::error!(
                error = ?error,
                "Transaction ingestion failed"
            );

            Err(ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: "transaction_storage_error",
                message: String::from("Transaction could not be stored"),
            })
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/cases", get(list_cases).post(create_case))
        .route("/cases/high-risk", get(list_high_risk_cases))
        .route(
            "/cases/{id}",
            get(get_case).patch(update_case).delete(delete_case),
        )
        .route("/cases/{id}/status", patch(update_case_status))
        .route("/cases/{id}/investigator", patch(assign_investigator))
        .route("/transactions", post(create_transaction))
        .route("/transactions/import-csv", post(import_transactions_csv))
        .route_layer(from_fn_with_state(state.jwt_secret.clone(), require_auth));

    Router::new()
        // Public routes
        .route("/health", get(health_check))
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/about", get(about))
        .route("/health/database", get(database_health))
        // Protected case routes
        .merge(protected_routes)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = %request.uri().path(),
                )
            }),
        )
        .with_state(state)
}
