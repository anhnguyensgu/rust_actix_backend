pub mod util {
    use crate::account::persistence::Account;
    use crate::error::AppError;

    pub enum JwtError {
        Failed,
    }
    pub fn generate_token(acc: &Account) -> Result<(String, String, i64), JwtError> {
        Ok(("".into(), "".into(), 0))
    }
    impl From<JwtError> for AppError {
        fn from(e: JwtError) -> Self {
            match e {
                JwtError::Failed => AppError::InternalError,
            }
        }
    }
}
