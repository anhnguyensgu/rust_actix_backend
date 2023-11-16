pub mod handler {
    use super::persistence;
    use crate::authentication::persistence::salt;
    use crate::error::AppError;
    use crate::{account, BB8Pool};

    use crate::jwt::util::JwtGenerator;
    use actix_web::web::{Data, Json};
    use actix_web::{post, Responder};
    use actix_web_httpauth::extractors::bearer::BearerAuth;
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use validator::Validate;

    #[derive(Serialize)]
    struct LoginResponse {
        pub access_token: String,
        pub refresh_token: String,
        pub expired_at: i64,
    }

    #[derive(Deserialize, Validate, Default)]
    #[serde(default)]
    pub struct LoginRequest {
        #[validate(length(min = 1, message = "username is required"))]
        pub username: String,

        #[validate(length(min = 1, message = "password is required"))]
        pub password: String,
    }

    #[post("/login")]
    pub async fn login(
        body: Json<LoginRequest>,
        pg_pool: Data<PgPool>,
        redis_pool: Data<BB8Pool>,
        generator: Data<JwtGenerator>,
    ) -> Result<impl Responder, AppError> {
        let LoginRequest { username, password } = body.into_inner();
        let salt = salt(&username, &pg_pool).await?;
        let hashed_password = sha256::digest(format!("{}{salt}", password));
        let user_id = persistence::login(&username, &hashed_password, &pg_pool).await?;
        let account = account::persistence::get_by_id(user_id, &pg_pool).await?;
        let response = generator.generate_token(&account).map(
            |(access_token, refresh_token, expired_at)| LoginResponse {
                access_token,
                refresh_token,
                expired_at,
            },
        )?;

        persistence::refresh_token::save(&redis_pool, &response.refresh_token, user_id).await?;

        Ok(Json(response))
    }

    #[post("/refresh")]
    pub async fn refresh(
        token: BearerAuth,
        redis_pool: Data<BB8Pool>,
        pg_pool: Data<PgPool>,
        generator: Data<JwtGenerator>,
    ) -> Result<impl Responder, AppError> {
        let user_id = persistence::refresh_token::get_then_del(&redis_pool, token.token()).await?;
        let account = account::persistence::get_by_id(user_id, &pg_pool).await?;
        let response = generator.generate_token(&account).map(
            |(access_token, refresh_token, expired_at)| LoginResponse {
                access_token,
                refresh_token,
                expired_at,
            },
        )?;

        persistence::refresh_token::save(&redis_pool, &response.refresh_token, user_id).await?;
        Ok(Json(response))
    }
}

pub mod persistence {
    use sqlx::{query, FromRow, PgPool, Row};

    #[derive(FromRow)]
    pub struct Credential {
        pub username: String,
        pub hashed_password: String,
        pub salt: String,
        pub user_id: i64,
    }

    pub async fn salt(username: &str, conn: &PgPool) -> sqlx::Result<String> {
        query("select salt from credentials where username = $1")
            .bind(username)
            .fetch_one(conn)
            .await
            .map(|r| r.get(0))
    }

    pub async fn login(username: &str, password: &str, conn: &PgPool) -> sqlx::Result<i64> {
        query("select user_id from credentials where username = $1 and hashed_password = $2")
            .bind(username)
            .bind(password)
            .fetch_one(conn)
            .await
            .map(|r| r.get(0))
    }

    pub mod refresh_token {
        use bb8_redis::redis::AsyncCommands;
        use tracing::error;

        use crate::{error::AppError, BB8Pool};

        pub async fn get_then_del(pool: &BB8Pool, refresh_token: &str) -> Result<i64, AppError> {
            let mut pool = pool.get().await.map_err(|e| {
                error!("get pool error {e:?}");
                AppError::InternalError
            })?;
            let key = keys(refresh_token);
            let Some(user_id): Option<i64> = pool.get(&key).await? else {
                return Err(AppError::Unauthorized);
            };
            pool.del(&key).await?;

            Ok(user_id)
        }

        pub async fn save(
            pool: &BB8Pool,
            refresh_token: &str,
            user_id: i64,
        ) -> Result<(), AppError> {
            let mut pool = pool.get().await.map_err(|e| {
                error!("get pool error {e:?}");
                AppError::InternalError
            })?;
            pool.set_ex(keys(refresh_token), user_id, duration())
                .await?;
            Ok(())
        }

        fn keys(refresh_token: &str) -> String {
            format!("refresh_token:{}", refresh_token)
        }

        fn duration() -> usize {
            24 * 60 * 60 * 14
        }
    }
}
