use crate::{error, iam::Claims, Error, Json, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::users::{self, Class};
use sea_orm::{DbErr, EntityTrait, Set, RuntimeErr};
use serde::Deserialize;
use sqlx::Error as SqlxError;

#[derive(Deserialize)]
pub struct Request {
    pub school: String,
    pub class: Class,
}

pub async fn register<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let user = users::ActiveModel {
        id: Set(claims.subject.clone()),
        school: Set(request.school),
        class: Set(request.class),
        ..Default::default()
    };

    let result = users::Entity::insert(user).exec(shared.db()).await;

    match result {
        Err(DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(error)))) => {
            if error.message() == "duplicate key value violates unique constraint \"users_pkey\"" {
                Err(error::USER_ALREADY_EXISTS)
            } else {
                Err(Error::internal(error))
            }
        },
        Err(error) => Err(Error::internal(error)),
        Ok(_) => Ok(StatusCode::CREATED),
    }
}
