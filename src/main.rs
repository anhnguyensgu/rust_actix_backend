use crate::jwt::util::JwtGenerator;
use crate::middleware::RequestContext;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{scope, Data};
use actix_web::{App, HttpMessage, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use bb8_redis::bb8::Pool;
use bb8_redis::redis::RedisError;
use bb8_redis::RedisConnectionManager;
use error::AppError;
use sqlx::PgPool;
use tracing::error;
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

    let redis_url = std::env::var("REDIS_URL").expect("should have redis_url");
    let redis_pool = create_redis_pool(&redis_url).await.unwrap();

    let secret_key = std::env::var("SECRET_KEY").expect("secret is required");
    let generator = Data::new(JwtGenerator::new(&secret_key));
    let pool = Data::new(pool);
    let redis_pool = Data::new(redis_pool);

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(|req, cre| async move {
            let token = cre.token();
            let Some(Ok(token_data)) = req
                .app_data::<Data<JwtGenerator>>()
                .map(|jwt_manager| jwt_manager.verify(token))
            else {
                return Err((ErrorUnauthorized(AppError::Unauthorized), req));
            };

            let context = RequestContext {
                user_id: token_data.claims.sub,
            };
            req.extensions_mut().insert(context);
            Ok(req)
        });

        App::new()
            .app_data(pool.clone())
            .app_data(redis_pool.clone())
            .app_data(generator.clone())
            .service(
                scope("/api/v1")
                    .service(
                        scope("/auth")
                            .service(authentication::handler::login)
                            .service(authentication::handler::refresh),
                    )
                    .service(scope("/accounts").service(account::handler::register))
                    .service(
                        scope("/assessments")
                            .wrap(auth)
                            .service(assessment::handler::get_all)
                            .service(assessment::handler::get_one)
                            .service(assessment::handler::create)
                            .service(assessment::handler::update),
                    ),
            )
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}

pub type BB8Pool = Pool<RedisConnectionManager>;

pub async fn create_redis_pool(host_addr: &str) -> Result<BB8Pool, AppError> {
    let manager = RedisConnectionManager::new(host_addr)?;
    let pool = Pool::builder().build(manager).await?;
    Ok(pool)
}

impl From<RedisError> for AppError {
    fn from(e: RedisError) -> Self {
        error!("redis error {e:?}");
        AppError::InternalError
    }
}
