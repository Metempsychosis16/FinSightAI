use crate::models::user::User;
use crate::services::password_service::verify_password;
use sqlx::PgPool;

#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    Database(sqlx::Error),
}

pub async fn authenticate_user(
    db: &PgPool,
    email: &str,
    password: &str,
) -> Result<User, LoginError> {
    let email = email.trim().to_lowercase();

    if email.is_empty() || email.len() > 254 || email.matches('@').count() != 1 {
        return Err(LoginError::InvalidCredentials);
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT
            id,
            email,
            password_hash,
            role,
            is_active,
            created_at,
            updated_at
        FROM users
        WHERE email = $1
        "#,
    )
    .bind(&email)
    .fetch_optional(db)
    .await
    .map_err(LoginError::Database)?
    .ok_or(LoginError::InvalidCredentials)?;

    if !user.is_active {
        return Err(LoginError::InvalidCredentials);
    }

    match verify_password(password, &user.password_hash) {
        Ok(true) => Ok(user),
        Ok(false) | Err(_) => Err(LoginError::InvalidCredentials),
    }
}
