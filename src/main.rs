use crate::jwt::util::JwtGenerator;
use actix_web::web::{scope, Data};
use actix_web::{App, HttpServer};
use sqlx::PgPool;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

mod account;
mod assessment;
mod authentication;
mod error;
mod jwt;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("should have database_url");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("should connect to database");

    let secret_key = std::env::var("SECRET_KEY").expect("secret is required");
    let generator = Data::new(JwtGenerator::new(&secret_key));
    let pool = Data::new(pool);
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .app_data(generator.clone())
            .service(scope("/auth").service(authentication::handler::login))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
