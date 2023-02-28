use axum::{extract::State, http::StatusCode, response::IntoResponse};
use entity::problems;
use sea_orm::{EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::Result, json::Json, StateTrait};

#[derive(Deserialize)]
pub struct Request {
    body: String,
    solution: i64,
    image: Option<String>,
}

#[derive(Serialize)]
pub struct Response {
    id: Uuid,
    body: String,
    solution: i64,
    image: Option<String>,
}

pub async fn create_problem<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<impl IntoResponse> {
    // TODO: permission check through the iam

    let problem = problems::ActiveModel {
        id: Set(Uuid::new_v4()),
        body: Set(request.body.clone()),
        solution: Set(request.solution.clone()),
        image: Set(request.image.clone()),
    };

    let res = problems::Entity::insert(problem).exec(state.db()).await?;

    Ok((
        StatusCode::CREATED,
        Json(Response {
            id: res.last_insert_id,
            body: request.body,
            solution: request.solution,
            image: request.image,
        }),
    ))
}
