use crate::{Json, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::users::{self, Class};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Request {
    pub id: String,
    pub school: String,
    pub class: Class,
}

pub async fn register<S: SharedTrait>(
    Extension(shared): Extension<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let user = users::ActiveModel {
        id: Set(request.id),
        school: Set(request.school),
        class: Set(request.class),
    };

    users::Entity::insert(user).exec(shared.db()).await?;

    Ok(StatusCode::CREATED)
}
