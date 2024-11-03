use crate::{
    error::{self, Result},
    extractors::Json,
    StateTrait,
};
use axum::extract::{Path, State};
use entity::problems;
use sea_orm::{EntityTrait, FromQueryResult};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, FromQueryResult)]
pub struct Response {
    id: Uuid,
    body: String,
    solution: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

pub async fn get_problem<S: StateTrait>(
    State(state): State<S>,
    Path(id): Path<String>,
) -> Result<Json<Response>> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return Err(error::PROBLEM_NOT_FOUND);
    };

    let res = problems::Entity::find_by_id(uuid)
        .into_model::<Response>()
        .one(state.db())
        .await?;

    let Some(problem) = res else {
        return Err(error::PROBLEM_NOT_FOUND);
    };

    Ok(Json(problem))
}

pub async fn list_problems<S: StateTrait>(State(state): State<S>) -> Result<Json<Vec<Response>>> {
    let res = problems::Entity::find()
        .into_model::<Response>()
        .all(state.db())
        .await?;

    Ok(Json(res))
}
