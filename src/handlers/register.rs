use crate::{error, error::DatabaseError, iam::Claims, Json, Result, StateTrait};
use axum::{http::StatusCode, Extension};
use entity::users::{self, Class};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Request {
    pub school: String,
    pub class: Class,
}

pub async fn register<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let user = users::ActiveModel {
        id: Set(claims.subject),
        school: Set(request.school),
        class: Set(request.class),
    };

    let result = users::Entity::insert(user)
        .exec_without_returning(state.db())
        .await;

    match result {
        Err(err) if err.unique_violation("PK_users") => return Err(error::USER_ALREADY_EXISTS),
        r => r?,
    };

    Ok(StatusCode::CREATED)
}
