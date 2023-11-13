pub mod persistence {
    use sqlx::{query, query_as, FromRow, PgPool};

    #[derive(FromRow)]
    pub struct Account {
        pub id: i64,
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
}
