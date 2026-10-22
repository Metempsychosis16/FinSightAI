use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

#[derive(Debug)]
pub enum PasswordServiceError {
    HashingFailed,
    InvalidStoredHash,
}
pub fn hash_password(password: &str) -> Result<String, PasswordServiceError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| PasswordServiceError::HashingFailed)?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, PasswordServiceError> {
    let parsed_hash =
        PasswordHash::new(stored_hash).map_err(|_| PasswordServiceError::InvalidStoredHash)?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(_) => Err(PasswordServiceError::InvalidStoredHash),
    }
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn hashes_password_without_storing_plaintext() {
        let password = "CorrectHorseExample!42";

        let hash = hash_password(password).expect("password hashing should succeed");

        assert_ne!(hash, password);
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn verifies_correct_password() {
        let password = "CorrectHorseExample!42";

        let hash = hash_password(password).expect("password hashing should succeed");

        assert_eq!(verify_password(password, &hash).unwrap(), true);
    }

    #[test]
    fn rejects_incorrect_password() {
        let hash =
            hash_password("CorrectHorseExample!42").expect("password hashing should succeed");

        assert_eq!(verify_password("WrongPassword!42", &hash).unwrap(), false);
    }

    #[test]
    fn different_salts_produce_different_hashes() {
        let password = "CorrectHorseExample!42";

        let first = hash_password(password).unwrap();
        let second = hash_password(password).unwrap();

        assert_ne!(first, second);

        assert!(verify_password(password, &first).unwrap());
        assert!(verify_password(password, &second).unwrap());
    }

    #[test]
    fn rejects_malformed_stored_hash() {
        let result = verify_password("CorrectHorseExample!42", "this-is-not-a-valid-argon2-hash");

        assert!(result.is_err());
    }
}
