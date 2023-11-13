pub mod handler {
    use super::persistence;
    use crate::account;
    use crate::error::AppError;

    use crate::jwt::util::generate_token;
    use actix_web::web::{Data, Json};
    use actix_web::{post, Responder};
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;

    #[derive(Serialize)]
    struct LoginResponse {
        pub access_token: String,
        pub refresh_token: String,
        pub expired_at: i64,
    }

    #[derive(Deserialize)]
    pub struct LoginRequest {
        pub username: String,
        pub password: String,
    }

    #[post("/login")]
    pub async fn login(
        body: Json<LoginRequest>,
        pg_pool: Data<PgPool>,
    ) -> Result<impl Responder, AppError> {
        let LoginRequest { username, password } = body.into_inner();
        let user_id = persistence::login(&username, &password, &pg_pool).await?;
        let account = account::persistence::get_by_id(user_id, &pg_pool).await?;
        let response =
            generate_token(&account).map(|(access_token, refresh_token, expired_at)| {
                LoginResponse {
                    access_token,
                    refresh_token,
                    expired_at,
                }
            })?;

        Ok(Json(response))
    }
}

pub mod persistence {
    use sqlx::{query, PgPool, Row};

    pub async fn login(username: &str, password: &str, conn: &PgPool) -> sqlx::Result<i64> {
        query("select user_id from credentials where username = $1 and hashed_password = $2")
            .bind(username)
            .bind(password)
            .fetch_one(conn)
            .await
            .map(|r| r.get(0))
    }
}
