use crate::{error, error::Error, handlers::socket::Event, iam::Claims, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, FromQueryResult, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use std::time::Duration;

#[derive(Debug, FromQueryResult)]
struct Team {
    locked: bool,
}

pub async fn leave_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = shared.db().begin().await?;

    let user = users::Entity::find_by_id(claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            tracing::warn!("tried to leave team without registration");
            error::USER_NOT_REGISTERED
        })?;

    if user.team.is_none() {
        return Err(error::USER_NOT_IN_TEAM);
    }

    let team = users::Entity::select_team(&user.id)
        // NOTE: maybe not neccessary because locking the team (in the application and not in the database)
        //       while this handler is running shouldn't make invalid state in the database
        .lock_shared()
        .select_only()
        .column(teams::Column::Locked)
        .into_model::<Team>()
        .one(&txn)
        .await?;

    if let Some(team) = team {
        if team.locked {
            return Err(error::LOCKED_TEAM);
        }

        let kafka_payload = serde_json::to_string(&Event::LeaveTeam {
            user: user.id.clone(),
        })
        .unwrap();
        let kafka_topic = super::get_kafka_topic(user.team.as_ref().unwrap());

        let mut active_model = user.into_active_model();
        active_model.team = Set(None);

        users::Entity::update(active_model).exec(&txn).await?;

        shared
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

        Ok(StatusCode::OK)
    } else {
        // wtf?
        Err(error::USER_NOT_IN_TEAM)
    }
}
