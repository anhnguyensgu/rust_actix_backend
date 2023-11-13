use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use derive_more::{Display, Error};
use tracing::error;

#[derive(Debug, Display, Error)]
pub enum AppError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,

    #[display(fmt = "Resource Not Found")]
    NotFound,
}

impl error::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadClientData => StatusCode::BAD_REQUEST,
            AppError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        error!("{e:?}");
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound,
            _ => AppError::InternalError,
        }
    }
}
