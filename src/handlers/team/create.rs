use crate::{
    error::{self, DatabaseError, Error, Result, ToPgError},
    iam::Claims,
    utils::generate_join_code,
    Json, SharedTrait, ValidatedJson,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::admin::{AdminOptions, NewTopic, TopicReplication};
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
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
    let txn = shared.db().begin().await?;

    let user = users::Entity::find_by_id(claims.subject.clone())
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            tracing::warn!("tried to create team without registration");
            error::USER_NOT_REGISTERED
        })?;

    if user.team.is_some() {
        return Err(error::ALREADY_IN_TEAM);
    }

    let id = Uuid::new_v4()
        .hyphenated()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned();

    let team = teams::ActiveModel {
        id: Set(id.clone()),
        name: Set(request.name),
        owner: Set(claims.subject.clone()),
        locked: Set(false),
        ..Default::default()
    };

    for _ in 0..16 {
        let model = {
            let mut model = team.clone();
            model.join_code = Set(generate_join_code(&mut shared.rng()));
            model
        };

        let result = teams::Entity::insert(model).exec(&txn).await;

        return match result.map_err(ToPgError::to_pg_error) {
            Err(Ok(pg_error)) => {
                if pg_error.unique_violation("teams_name_key") {
                    Err(error::DUPLICATE_TEAM_NAME)
                } else if pg_error.unique_violation("join_code_key") {
                    continue;
                } else {
                    Err(Error::internal(pg_error))
                }
            }
            Err(Err(error)) => Err(Error::internal(error)),
            Ok(_) => {
                let mut active_model = user.into_active_model();
                active_model.team = Set(Some(id.clone()));

                users::Entity::update(active_model).exec(&txn).await?;

                shared
                    .kafka_admin()
                    .create_topics(
                        &[NewTopic::new(
                            &super::get_kafka_topic(&id),
                            1,
                            TopicReplication::Fixed(1),
                        )],
                        &AdminOptions::new(),
                    )
                    .await
                    .map_err(Error::internal)?;

                txn.commit().await?;

                Ok((StatusCode::CREATED, Json(Response { id })))
            }
        };
    }

    Err(error::FAILED_TO_GENERATE_JOIN_CODE)
}
