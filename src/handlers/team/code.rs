use crate::{
    error::{self, DatabaseError, Error, Result, ToPgError},
    handlers::socket::Event,
    iam::Claims,
    utils, StateTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use std::time::Duration;

pub async fn regenerate_code<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = users::Entity::select_team(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != claims.subject && team.coowner.as_ref() != Some(&claims.subject) {
        return Err(error::USER_NOT_COOWNER);
    }

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    let kafka_topic = super::get_kafka_topic(&team.id);
    let model = team.into_active_model();

    for _ in 0..16 {
        let new_code = utils::generate_join_code(&mut state.rng());

        let mut model = model.clone();
        model.join_code = Set(new_code.clone());

        let res = teams::Entity::update(model).exec(&txn).await;

        return match res.map_err(ToPgError::to_pg_error) {
            Err(Ok(pg_error)) => {
                if pg_error.unique_violation("join_code_key") {
                    continue;
                }
                Err(Error::internal(pg_error))
            }
            Err(Err(error)) => Err(Error::internal(error)),
            Ok(_) => {
                let kafka_payload = serde_json::to_string(&Event::UpdateTeam {
                    name: None,
                    owner: None,
                    coowner: None,
                    locked: None,
                    code: Some(new_code),
                })
                .unwrap();

                state
                    .kafka_producer()
                    .send(
                        FutureRecord::<(), String>::to(&kafka_topic)
                            .partition(0)
                            .payload(&kafka_payload),
                        Duration::from_secs(5),
                    )
                    .await
                    .map_err(|(err, _)| Error::internal(err))?;

                txn.commit().await?;
                Ok(StatusCode::NO_CONTENT)
            }
        };
    }

    Err(error::FAILED_TO_GENERATE_JOIN_CODE)
}
