pub mod handler {
    use crate::error::AppError;
    use crate::middleware::RequestContext;
    use actix_web::web::{Data, Json};
    use actix_web::{
        get, patch, post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Result,
    };
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use tracing::debug;
    use validator::Validate;

    #[derive(Serialize, Deserialize)]
    pub struct ResTemp;

    #[get("/")]
    pub async fn get_all(
        pool: Data<PgPool>,
        context_data: Option<web::ReqData<RequestContext>>,
    ) -> Result<impl Responder, AppError> {
        let user_id = context_data
            .ok_or_else(|| AppError::Unauthorized)
            .map(|req_context| req_context.user_id)?;
        let assessments = super::persistence::get_by_user(user_id, &pool).await?;
        Ok(Json(assessments))
    }

    #[derive(Deserialize, Validate, Default)]
    #[serde(default)]
    pub struct AssessmentCreationRequest {
        pub topics: Vec<String>,
    }

    #[post("/")]
    pub async fn create(
        pool: Data<PgPool>,
        context_data: Option<web::ReqData<RequestContext>>,
    ) -> Result<impl Responder, AppError> {
        let user_id = context_data
            .ok_or_else(|| AppError::Unauthorized)
            .map(|req_context| req_context.user_id)?;

        let assessments = super::persistence::get_by_user(user_id, &pool).await?;
        Ok(Json(assessments))
    }

    #[get("/{assessement_id}")]
    pub async fn get_one(
        assessment_id: web::Path<i64>,
        pool: Data<PgPool>,
        context_data: Option<web::ReqData<RequestContext>>,
    ) -> Result<impl Responder, AppError> {
        let user_id = context_data
            .ok_or_else(|| AppError::Unauthorized)
            .map(|req_context| req_context.user_id)?;
        debug!("getting assessment for {user_id}");

        let assessment =
            super::persistence::get_by_assessment_id(assessment_id.into_inner(), user_id, &pool)
                .await?;

        Ok(Json(assessment))
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "snake_case")]
    pub enum AssessmentAtrritbutes {
        Started(bool),
        Finished(bool),
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PathObject {
        assessment_id: i64,
    }

    #[patch("{assessment_id}")]
    pub async fn update(
        request: HttpRequest,
        path_params: web::Path<PathObject>,
        body: Json<AssessmentAtrritbutes>,
        pool: Data<PgPool>,
    ) -> Result<impl Responder, AppError> {
        use super::persistence::{update_end_time, update_start_time};

        let user_id = request
            .extensions()
            .get::<RequestContext>()
            .ok_or_else(|| AppError::Unauthorized)
            .map(|&RequestContext { user_id }| user_id)?;
        match body.into_inner() {
            AssessmentAtrritbutes::Started(_) => {
                update_start_time(path_params.assessment_id, user_id, &pool).await?;
            }
            AssessmentAtrritbutes::Finished(_) => {
                update_end_time(path_params.assessment_id, user_id, &pool).await?;
            }
        }
        Ok(HttpResponse::Created())
    }
}

pub mod persistence {
    use chrono::Utc;
    use serde::{Deserialize, Serialize};
    use sqlx::{query, FromRow, PgPool};

    #[derive(FromRow, Deserialize, Serialize)]
    pub struct Assessment {
        pub id: i64,
        pub user_id: i64,
        pub started_at: Option<i32>,
        pub updated_at: Option<i32>,
    }

    pub async fn get_by_user(user_id: i64, conn: &PgPool) -> sqlx::Result<Vec<Assessment>> {
        sqlx::query_as!(
            Assessment,
            "select id, user_id, started_at, updated_at from assessments where user_id = $1",
            user_id
        )
        .fetch_all(conn)
        .await
    }

    pub(crate) async fn get_by_assessment_id(
        assessment_id: i64,
        user_id: i64,
        pool: &PgPool,
    ) -> sqlx::Result<Assessment> {
        sqlx::query_as!(
            Assessment,
            "select id, user_id, started_at, updated_at from assessments where user_id = $1 and id = $2",
            user_id, assessment_id
        ).fetch_one(pool).await
    }

    pub(crate) async fn update_start_time(
        assessment_id: i64,
        user_id: i64,
        pool: &PgPool,
    ) -> sqlx::Result<u64> {
        query!(
            "update assessments set started_at = $1, updated_at = $1 where id = $2 and user_id = $3 and started_at is null",
            Utc::now().timestamp() as i32,
            assessment_id,
            user_id
        )
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
    }

    pub(crate) async fn update_end_time(
        assessment_id: i64,
        user_id: i64,
        pool: &PgPool,
    ) -> sqlx::Result<u64> {
        query!(
            "update assessments set finished_at = $1, updated_at = $1 where id = $2 and user_id = $3 and finished_at is null",
            Utc::now().timestamp() as i32,
            assessment_id,
            user_id
        )
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
    }
}
