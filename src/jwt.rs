pub mod util {
    use std::ops::Add;

    use crate::account::persistence::Account;
    use crate::error::AppError;
    use chrono::Utc;
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tracing::error;

    pub enum JwtError {
        Failed,
    }

    pub struct JwtGenerator {
        encode_key: EncodingKey,
        decode_key: DecodingKey,
        algo: Algorithm,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub exp: usize,
        pub sub: i64,     // Optional. Subject (whom token refers to)
        pub name: String, // Optional. Subject (whom token refers to)
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
            let expired_at = Utc::now().add(chrono::Duration::days(7)).timestamp();
            let payload = json!({
                "sub": acc.id,
                "name": acc.first_name,
                "exp": expired_at,
            });
            let header = Header::new(self.algo);
            let token = encode(&header, &payload, &self.encode_key).map_err(|e| {
                error!("error encoding {e:?}");
                JwtError::Failed
            })?;

            let session_id: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();

            let refresh_token = sha256::digest(format!("{}{session_id}", acc.id));

            Ok((token, refresh_token, expired_at))
        }

        pub fn verify(
            &self,
            token: &str,
        ) -> Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> {
            decode::<Claims>(token, &self.decode_key, &Validation::default())
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
