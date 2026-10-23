use axum::{
    extract::State,
    http::{Request, StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::{
    auth::{jwt::verify_token, policy::is_authorized},
    models::role::Role,
};

pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = verify_token(token, &secret).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let role = Role::parse(&claims.role).ok_or(StatusCode::UNAUTHORIZED)?;

    if !is_authorized(role, request.method(), request.uri().path()) {
        return Err(StatusCode::FORBIDDEN);
    }

    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
