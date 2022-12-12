use axum::http::StatusCode;
use axum::Extension;
use entity::problems;
use sea_orm::{EntityTrait, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::Result, json::Json, StateTrait};

#[derive(Deserialize)]
pub struct Request {
    body: String,
    solution: i64,
    image: Option<String>,
}

pub async fn create_problem<S: StateTrait>(
    Extension(state): Extension<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    // TODO: permission check through the iam

    let problem = problems::ActiveModel {
        id: Set(Uuid::new_v4()),
        body: Set(request.body),
        solution: Set(request.solution),
        image: Set(request.image),
    };

    problems::Entity::insert(problem).exec(state.db()).await?;

    Ok(StatusCode::CREATED)
}
