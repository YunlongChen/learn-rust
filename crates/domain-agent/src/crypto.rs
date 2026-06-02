//! Cryptographic utilities

use rand::Rng;
use sha2::{Sha256, Digest};

/// Hash a key using SHA-256
pub fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate a random key
pub fn generate_key() -> String {
    let key: [u8; 32] = rand::thread_rng().gen();
    base64::encode(key)
}

mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(data: impl AsRef<[u8]>) -> String {
        let bytes = data.as_ref();
        let mut result = String::with_capacity(bytes.len() * 2);
        for &byte in bytes {
            result.push(HEX_CHARS[(byte >> 4) as usize] as char);
            result.push(HEX_CHARS[(byte & 0xf) as usize] as char);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_key() {
        let hash = hash_key("test_key");
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }
}
