use crate::{
    error::{self, Error, Result},
    handlers::socket::Event,
    iam::Claims,
    json::Json,
    StateTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct Request {
    user: String,
}

pub async fn kick_user<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
    Json(request): Json<Request>,
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

    if request.user == team.owner {
        return Err(error::CANNOT_KICK_OWNER);
    }

    if request.user == claims.subject {
        return Err(error::CANNOT_KICK_THEMSELF);
    }

    let user = users::Entity::find_by_id(request.user.clone())
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::NO_SUCH_MEMBER)?;

    if user.team.as_ref() != Some(&team.id) {
        return Err(error::NO_SUCH_MEMBER);
    }

    let kafka_topic = super::get_kafka_topic(&team.id);

    if Some(&request.user) == team.coowner.as_ref() {
        let mut model = team.into_active_model();
        model.coowner = Set(None);
        teams::Entity::update(model).exec(&txn).await?;
    }

    let mut model = user.into_active_model();
    model.team = Set(None);

    users::Entity::update(model).exec(&txn).await?;

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&kafka_topic)
                .partition(0)
                .payload(&serde_json::to_string(&Event::KickUser { user: request.user }).unwrap()),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(err, _)| Error::internal(err))?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
