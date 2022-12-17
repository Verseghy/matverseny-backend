use crate::{
    error::{self, Result},
    json::Json,
    StateTrait,
};
use axum::{extract::Path, Extension};
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
    Extension(state): Extension<S>,
    Path(id): Path<String>,
) -> Result<Json<Response>> {
    // TODO: permission check through the iam

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

pub async fn list_problems<S: StateTrait>(
    Extension(state): Extension<S>,
) -> Result<Json<Vec<Response>>> {
    // TODO: permission check through the iam

    let res = problems::Entity::find()
        .into_model::<Response>()
        .all(state.db())
        .await?;

    Ok(Json(res))
}
