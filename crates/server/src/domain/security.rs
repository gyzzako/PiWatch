use pbkdf2::pbkdf2;
use pbkdf2::hmac::Hmac;
use sha2::Sha256;
use rand::RngCore;
use base64::Engine;
use subtle::ConstantTimeEq;

pub(crate) struct CryptoService;

impl CryptoService {
    pub fn generate_secret() -> String {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    }

    pub fn generate_salt() -> String {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    pub fn hash_with_salt(token: &str, salt_hex: &str) -> String {
        let salt = hex::decode(salt_hex).expect("invalid salt");
        let iterations = 100_000u32;
        let mut output = [0u8; 32];
        
        pbkdf2::<Hmac<Sha256>>(token.as_bytes(), &salt, iterations, &mut output)
            .expect("PBKDF2 failed");
        hex::encode(output)
    }

    pub fn verify(token: &str, salt_hex: &str, expected_hash: &str) -> bool {
        let computed = Self::hash_with_salt(token, salt_hex);
        let expected_bytes = expected_hash.as_bytes();
        let computed_bytes = computed.as_bytes();
        
        if computed_bytes.len() != expected_bytes.len() {
            return false;
        }
        
        ConstantTimeEq::ct_eq(computed_bytes, expected_bytes).into()
    }
}
