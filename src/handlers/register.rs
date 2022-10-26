use crate::{
    error,
    error::{DatabaseError, Error, ToPgError},
    iam::Claims,
    Json, Result, SharedTrait,
};
use axum::{http::StatusCode, Extension};
use entity::users::{self, Class};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;

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

    match result.map_err(ToPgError::to_pg_error) {
        Err(Ok(pg_error)) => {
            if pg_error.unique_violation("users_pkey") {
                Err(error::USER_ALREADY_EXISTS)
            } else {
                Err(Error::internal(pg_error))
            }
        }
        Err(Err(error)) => Err(Error::internal(error)),
        Ok(_) => Ok(StatusCode::CREATED),
    }
}
