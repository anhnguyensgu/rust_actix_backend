pub mod handler {
    use crate::middleware::RequestContext;
    use actix_web::web::{Data, Json};
    use actix_web::{get, post, put, web, HttpMessage, HttpRequest, Responder, Result};
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;

    #[derive(Serialize, Deserialize)]
    pub struct ResTemp;

    #[get("/")]
    pub async fn get_by_user(request: HttpRequest, pool: Data<PgPool>) -> Result<impl Responder> {
        let Some(&RequestContext { user_id }) = request.extensions().get::<RequestContext>() else {
            return Ok(Json(vec![]));
        };
        let Ok(assessments) = super::persistence::get_by_user(user_id, &pool).await else {
            return Ok(Json(vec![]));
        };
        Ok(Json(assessments))
    }

    #[post("/")]
    pub async fn create(name: web::Path<String>) -> impl Responder {
        format!("Hello {name}!")
    }

    #[put("/{assessment_id}")]
    pub async fn update(assessment_id: web::Path<String>) -> impl Responder {
        format!("Hello {assessment_id}!")
    }
}

pub mod persistence {
    use serde::{Deserialize, Serialize};
    use sqlx::{query_as, FromRow, PgPool};

    #[derive(FromRow, Deserialize, Serialize)]
    pub struct Assessment {
        pub id: i64,
    }

    pub async fn get_by_user(_user_id: i64, conn: &PgPool) -> sqlx::Result<Vec<Assessment>> {
        query_as::<_, Assessment>("").fetch_all(conn).await
    }
}
