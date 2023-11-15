pub mod handler {
    extern crate rand;
    use crate::{
        account::persistence::{create_account, NewAccount},
        error::AppError,
    };
    use actix_web::{post, web::Data, Responder};
    use actix_web_validator::Json;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use tracing::info;
    use validator::Validate;

    use super::persistence::Account;

    #[derive(Deserialize, Validate, Default)]
    #[serde(default)]
    pub struct AccountCreationRequest {
        #[validate(length(min = 1, message = "username is required"))]
        pub username: String,
        #[validate(length(min = 1, message = "password is required"))]
        pub password: String,
        #[validate(email)]
        pub email: String,
        #[validate(length(min = 1, message = "first_name is required"))]
        pub first_name: String,
        #[validate(length(min = 1, message = "last_name is required"))]
        pub last_name: String,
    }

    #[derive(Serialize)]
    pub struct AccountResponse {
        pub email: String,
        pub first_name: String,
        pub last_name: String,
    }

    impl From<Account> for AccountResponse {
        fn from(
            Account {
                email,
                first_name,
                last_name,
                ..
            }: Account,
        ) -> Self {
            Self {
                email,
                first_name,
                last_name,
            }
        }
    }

    #[post("")]
    pub async fn register(
        body: Json<AccountCreationRequest>,
        pool: Data<PgPool>,
    ) -> Result<impl Responder, AppError> {
        let salt: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let hashed_password = sha256::digest(format!("{}{salt}", body.password));
        info!("{hashed_password}");
        let new_account = NewAccount {
            email: body.email.clone(),
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            password: hashed_password,
            salt,
            username: body.username.clone(),
        };

        let new_account = create_account(new_account, &pool).await?;

        Ok(actix_web::web::Json(AccountResponse::from(new_account)))
    }
}
pub mod persistence {
    use serde::{Deserialize, Serialize};
    use sqlx::{query, query_as, FromRow, PgPool};

    #[derive(FromRow, Serialize, Deserialize)]
    pub struct Account {
        pub id: i64,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(FromRow, Serialize, Deserialize)]
    pub struct NewAccount {
        pub username: String,
        pub password: String,
        pub salt: String,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
    }

    pub async fn get_by_id(id: i64, conn: &PgPool) -> sqlx::Result<Account> {
        query_as::<_, Account>(
            "select id, email, first_name, last_name from accounts where id = $1",
        )
        .bind(id)
        .fetch_one(conn)
        .await
    }

    pub async fn create_account(new_account: NewAccount, conn: &PgPool) -> sqlx::Result<Account> {
        let mut tx = conn.begin().await?;
        let account = query_as::<_, Account>(
            "insert into accounts(email, first_name, last_name) values ($1, $2, $3) returning id, email, first_name, last_name",
        )
        .bind(new_account.email)
        .bind(new_account.first_name)
        .bind(new_account.last_name)
        .fetch_one(&mut *tx)
        .await?;

        query("insert into credentials(username, hashed_password, salt, user_id) values ($1, $2, $3, $4)",)
        .bind(new_account.username)
        .bind(new_account.password)
        .bind(new_account.salt)
        .bind(account.id)
        .execute(&mut *tx).await?;
        tx.commit().await?;

        Ok(account)
    }
}
