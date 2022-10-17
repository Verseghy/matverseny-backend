use crate::{error, handlers::socket::Event, iam::Claims, Error, Json, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, FromQueryResult, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use std::time::Duration;

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
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = shared.db().begin().await?;

    let team = teams::Entity::find_by_join_code(&request.code)
        .select_only()
        .column(teams::Column::Id)
        .column(teams::Column::Locked)
        .into_model::<Team>()
        .one(&txn)
        .await?;

    if let Some(team) = team {
        if team.locked {
            return Err(error::LOCKED_TEAM);
        }

        let user = users::Entity::find_by_id(claims.subject)
            .one(&txn)
            .await?
            .ok_or_else(|| {
                // this is suspicious so log it
                tracing::warn!("tried to join team without registration");
                error::USER_NOT_REGISTERED
            })?;

        if user.team.is_some() {
            return Err(error::ALREADY_IN_TEAM);
        }

        let kafka_payload = serde_json::to_string(&Event::JoinTeam {
            user: user.id.clone(),
        })
        .unwrap();

        let mut active_model = user.into_active_model();
        active_model.team = Set(Some(team.id.clone()));

        users::Entity::update(active_model).exec(&txn).await?;

        shared
            .kafka_producer()
            .send(
                FutureRecord::<(), String>::to(&super::get_kafka_topic(&team.id))
                    .partition(0)
                    .payload(&kafka_payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| Error::internal(err))?;

        txn.commit().await?;

        Ok(StatusCode::OK)
    } else {
        Err(error::JOIN_CODE_NOT_FOUND)
    }
}
