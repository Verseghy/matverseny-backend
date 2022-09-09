use crate::{error, iam::Claims, Json, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use sea_orm::{EntityTrait, FromQueryResult, IntoActiveModel, QuerySelect, Set};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Request {
    code: String,
}

#[derive(Debug, FromQueryResult)]
struct Team {
    id: String,
    locked: bool,
}

pub async fn join_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    Extension(claims): Extension<Arc<Claims>>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let team = teams::Entity::find_by_join_code(&request.code)
        .select_only()
        .column(teams::Column::Id)
        .column(teams::Column::Locked)
        .into_model::<Team>()
        .one(shared.db())
        .await?;

    if let Some(team) = team {
        if !team.locked {
            return Err(error::LOCKED_TEAM);
        }

        let user = users::Entity::find_by_id(claims.subject.clone())
            .one(shared.db())
            .await?
            .expect("user not in database?");

        if user.team.is_some() {
            return Err(error::ALREADY_IN_TEAM);
        }

        let mut active_model = user.into_active_model();
        active_model.team = Set(Some(team.id));

        users::Entity::update(active_model)
            .exec(shared.db())
            .await?;

        Ok(StatusCode::OK)
    } else {
        Err(error::JOIN_CODE_NOT_FOUND)
    }
}
