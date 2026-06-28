use sha2::{Sha256, Digest};
use rand::RngCore;

pub(crate) struct CryptoService;

impl CryptoService {
    pub fn generate_secret() -> String {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    pub fn generate_salt() -> String {
         let mut bytes = [0u8; 16];
         rand::rng().fill_bytes(&mut bytes);
         hex::encode(bytes)
    }

    pub fn hash_with_salt(token: &str, salt_hex: &str) -> String {
        let salt = Self::salt_bytes(salt_hex);

        let mut hasher = Sha256::new();
        hasher.update(&salt);
        hasher.update(token.as_bytes());

        hex::encode(hasher.finalize())
    }

    pub fn verify(token: &str, salt_hex: &str, expected_hash: &str) -> bool {
        Self::hash_with_salt(token, salt_hex) == expected_hash
    }

    fn salt_bytes(salt_hex: &str) -> Vec<u8> {
        hex::decode(salt_hex)
            .expect("invalid salt")
    }
}
