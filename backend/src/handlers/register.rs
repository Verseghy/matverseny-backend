use crate::{
    error::{self, DatabaseError},
    extractors::{Json, UserID},
    Result, StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::users::{self, constraints::*, Class};
use sea_orm::{EntityTrait, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Request {
    pub school: String,
    pub class: Class,
}

pub async fn register<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let user = users::ActiveModel {
        id: Set(*user_id),
        school: Set(request.school),
        class: Set(request.class),
    };

    let result = users::Entity::insert(user)
        .exec_without_returning(state.db())
        .await;

    match result {
        Err(err) if err.unique_violation(PK_USERS) => return Err(error::USER_ALREADY_EXISTS),
        r => r?,
    };

    Ok(StatusCode::CREATED)
}
