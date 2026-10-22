use crate::models::user::User;
use crate::services::password_service::hash_password;
use sqlx::PgPool;

#[derive(Debug)]
pub enum RegistrationError {
    InvalidEmail,
    WeakPassword,
    DuplicateEmail,
    PasswordHashingFailed,
    Database(sqlx::Error),
}

pub async fn register_user(
    db: &PgPool,
    email: &str,
    password: &str,
) -> Result<User, RegistrationError> {
    let email = email.trim().to_lowercase();

    // Basic input validation.
    if email.len() > 254
        || email.chars().any(char::is_whitespace)
        || email.matches('@').count() != 1
    {
        return Err(RegistrationError::InvalidEmail);
    }

    let Some((local, domain)) = email.split_once('@') else {
        return Err(RegistrationError::InvalidEmail);
    };

    if local.is_empty()
        || domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return Err(RegistrationError::InvalidEmail);
    }

    // Limit password size before the expensive hashing operation.
    if password.chars().count() < 12 || password.len() > 1024 {
        return Err(RegistrationError::WeakPassword);
    }

    let password_hash =
        hash_password(password).map_err(|_| RegistrationError::PasswordHashingFailed)?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (
            email,
            password_hash,
            role
        )
        VALUES ($1, $2, 'INVESTIGATOR')
        ON CONFLICT (email) DO NOTHING
        RETURNING
            id,
            email,
            password_hash,
            role,
            is_active,
            created_at,
            updated_at
        "#,
    )
    .bind(&email)
    .bind(&password_hash)
    .fetch_optional(db)
    .await
    .map_err(RegistrationError::Database)?;

    user.ok_or(RegistrationError::DuplicateEmail)
}
