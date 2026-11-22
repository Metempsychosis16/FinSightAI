use sha2::{Digest, Sha256};

pub fn calculate_sha256(data: &[u8]) -> String {
    let digest = Sha256::digest(data);

    format!("{digest:x}")
}

#[cfg(test)]
mod tests {
    use super::calculate_sha256;

    #[test]
    fn calculates_known_sha256_hash() {
        let hash = calculate_sha256(b"hello");

        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn produces_64_character_hash() {
        let hash = calculate_sha256(b"FinSight AI evidence");

        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn different_inputs_produce_different_hashes() {
        let first = calculate_sha256(b"evidence version 1");
        let second = calculate_sha256(b"evidence version 2");

        assert_ne!(first, second);
    }

    #[test]
    fn empty_input_has_valid_sha256_hash() {
        let hash = calculate_sha256(b"");

        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
