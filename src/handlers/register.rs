use crate::{iam::Claims, Json, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::users::{self, Class};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Request {
    pub school: String,
    pub class: Class,
}

pub async fn register<S: SharedTrait>(
    Extension(shared): Extension<S>,
    Extension(claims): Extension<Arc<Claims>>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let user = users::ActiveModel {
        id: Set(claims.subject.clone()),
        school: Set(request.school),
        class: Set(request.class),
        ..Default::default()
    };

    users::Entity::insert(user).exec(shared.db()).await?;

    Ok(StatusCode::CREATED)
}
