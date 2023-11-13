pub mod util {
    use crate::account::persistence::Account;
    use crate::error::AppError;
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tracing::info;

    pub enum JwtError {
        Failed,
    }

    pub struct JwtGenerator {
        encode_key: EncodingKey,
        decode_key: DecodingKey,
        algo: Algorithm,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        // aud: String,
        // // Optional. Audience
        exp: usize,
        // // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
        // iat: usize,
        // // Optional. Issued at (as UTC timestamp)
        // iss: String,
        // // Optional. Issuer
        // nbf: usize,
        // Optional. Not Before (as UTC timestamp)
        sub: i64,     // Optional. Subject (whom token refers to)
        name: String, // Optional. Subject (whom token refers to)
    }

    impl JwtGenerator {
        pub fn new(secret: &str) -> Self {
            Self {
                encode_key: EncodingKey::from_secret(secret.as_bytes()),
                decode_key: DecodingKey::from_secret(secret.as_bytes()),
                algo: Algorithm::HS256,
            }
        }

        pub fn generate_token(&self, acc: &Account) -> Result<(String, String, i64), JwtError> {
            let payload = json!({
                "sub": acc.id,
                "name": acc.first_name,
            });
            let header = Header::new(self.algo);
            let token = encode(&header, &payload, &self.encode_key).unwrap();
            Ok((token.clone(), token, 0))
        }

        pub fn verify(&self, token: &str) {
            let token = decode::<Claims>(token, &self.decode_key, &Validation::default());
            info!("{token:?}");
        }
    }

    impl From<JwtError> for AppError {
        fn from(e: JwtError) -> Self {
            match e {
                JwtError::Failed => AppError::InternalError,
            }
        }
    }
}
