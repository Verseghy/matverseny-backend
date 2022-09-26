use crate::{
    error::{self, Error, Result},
    iam::Claims,
    utils::generate_join_code,
    Json, SharedTrait, ValidatedJson,
};
use axum::{http::StatusCode, Extension};
use entity::teams;
use sea_orm::{DbErr, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Request {
    #[validate(length(max = 32))]
    name: String,
}

#[derive(Serialize)]
pub struct Response {
    id: String,
}

pub async fn create_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
    ValidatedJson(request): ValidatedJson<Request>,
) -> Result<(StatusCode, Json<Response>)> {
    let id = Uuid::new_v4()
        .hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned();

    let team = teams::ActiveModel {
        id: Set(id.clone()),
        name: Set(request.name),
        owner: Set(claims.subject),
        locked: Set(false),
        // TODO: handle clash
        join_code: Set(generate_join_code(&mut shared.rng().clone())),
        ..Default::default()
    };

    let result = teams::Entity::insert(team).exec(shared.db()).await;

    match result {
        Err(DbErr::Query(error)) => {
            if &error[..] == "error returned from database: duplicate key value violates unique constraint \"teams_name_key\"" {
                Err(error::DUPLICATE_TEAM_NAME)
            } else if &error[..] == "error returned from database: insert or update on table \"teams\" violates foreign key constraint \"FK_teams_owner\"" {
                // this is suspicious so log it
                tracing::warn!("tried to create team without registration");
                Err(error::USER_NOT_REGISTERED)
            } else {
                Err(Error::internal(error))
            }
        }
        Err(error) => Err(Error::internal(error)),
        Ok(_) => Ok((StatusCode::CREATED, Json(Response { id }))),
    }
}
