use crate::{
    error::{self, Result},
    json::Json,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::problems;
use sea_orm::EntityTrait;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    id: Uuid,
}

pub async fn delete_problem<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    // TODO: permission check through the iam

    let res = problems::Entity::delete_by_id(request.id)
        .exec(state.db())
        .await?;

    if res.rows_affected == 0 {
        return Err(error::PROBLEM_NOT_FOUND);
    }

    // TODO: update order if this problem was in the list.
    // TODO: send kafka events

    Ok(StatusCode::OK)
}
