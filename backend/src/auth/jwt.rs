use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    SigningFailed,
}

pub fn create_token(
    user_id: i64,
    role: &str,
    secret: &str,
    now: usize,
    lifetime_seconds: usize,
) -> Result<String, JwtError> {
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_owned(),
        iat: now,
        exp: now.saturating_add(lifetime_seconds),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| JwtError::SigningFailed)
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp", "sub"]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| JwtError::InvalidToken)
}

#[cfg(test)]
mod tests {
    use super::{create_token, verify_token};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_timestamp() -> usize {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_secs() as usize
    }

    #[test]
    fn verifies_a_valid_token() {
        let now = current_timestamp();

        let token = create_token(
            42,
            "INVESTIGATOR",
            "test-secret-with-at-least-32-bytes-long",
            now,
            900,
        )
        .unwrap();

        let claims = verify_token(&token, "test-secret-with-at-least-32-bytes-long").unwrap();

        assert_eq!(claims.sub, "42");
        assert_eq!(claims.role, "INVESTIGATOR");
    }

    #[test]
    fn rejects_a_token_signed_with_another_secret() {
        let now = current_timestamp();

        let token = create_token(
            42,
            "INVESTIGATOR",
            "test-secret-with-at-least-32-bytes-long",
            now,
            900,
        )
        .unwrap();

        assert!(verify_token(&token, "different-secret-with-at-least-32-bytes",).is_err());
    }

    #[test]
    fn rejects_an_expired_token() {
        let now = current_timestamp();

        let token = create_token(
            42,
            "INVESTIGATOR",
            "test-secret-with-at-least-32-bytes-long",
            now.saturating_sub(600),
            1,
        )
        .unwrap();

        assert!(verify_token(&token, "test-secret-with-at-least-32-bytes-long",).is_err());
    }
}
