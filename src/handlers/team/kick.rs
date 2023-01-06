use crate::{
    error::{self, Result},
    handlers::socket::Event,
    iam::Claims,
    json::Json,
    utils::topics,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{team_members, teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use std::time::Duration;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    user: Uuid,
}

pub async fn kick_user<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != claims.subject && team.co_owner.as_ref() != Some(&claims.subject) {
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

    let user = users::Entity::find_by_id(request.user)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::NO_SUCH_MEMBER)?;

    let res = team_members::Entity::delete(team_members::ActiveModel {
        user_id: Set(user.id),
        team_id: Set(team.id),
    })
    .exec(&txn)
    .await?;

    if res.rows_affected == 0 {
        return Err(error::NO_SUCH_MEMBER);
    }

    let kafka_topic = topics::team_info(&team.id);

    if Some(request.user) == team.co_owner {
        let mut model = team.into_active_model();
        model.co_owner = Set(None);
        teams::Entity::update(model).exec(&txn).await?;
    }

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&kafka_topic)
                .partition(0)
                .payload(&serde_json::to_string(&Event::KickUser { user: request.user }).unwrap()),
            Duration::from_secs(5),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
