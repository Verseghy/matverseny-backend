use crate::{
    error,
    error::Result,
    handlers::socket::Event,
    iam::Claims,
    utils::{set_option, topics},
    StateTrait, ValidatedJson,
};
use axum::{extract::State, http::StatusCode};
use entity::teams;
use rdkafka::producer::FutureRecord;
use sea_orm::{ConnectionTrait, EntityTrait, IntoActiveModel, QuerySelect, TransactionTrait};
use serde::Deserialize;
use std::time::Duration;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Request {
    #[validate(length(max = 32))]
    name: Option<String>,
    owner: Option<Uuid>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    co_owner: Option<Option<Uuid>>,
    locked: Option<bool>,
}

pub async fn update_team<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
    ValidatedJson(request): ValidatedJson<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != claims.subject {
        return Err(error::USER_NOT_OWNER);
    }

    // Allow updates when the team is locked, but the request sets it the be unlocked.
    // Otherwise error
    if (request.locked.is_none() || request.locked == Some(true)) && team.locked {
        return Err(error::LOCKED_TEAM);
    }

    // without this the ORM would generate an invalid sql statement
    if request.name.is_none()
        && request.owner.is_none()
        && request.co_owner.is_none()
        && request.locked.is_none()
    {
        return Ok(StatusCode::NO_CONTENT);
    }

    if let Some(owner) = &request.owner {
        if owner == &claims.subject {
            return Ok(StatusCode::NO_CONTENT);
        }

        if !is_user_in_team(&txn, owner, &team.id).await? {
            return Err(error::NO_SUCH_MEMBER);
        }
    }

    if let Some(Some(coowner)) = &request.co_owner {
        if !is_user_in_team(&txn, coowner, &team.id).await? {
            return Err(error::NO_SUCH_MEMBER);
        }
    }

    let kafka_payload = serde_json::to_string(&Event::UpdateTeam {
        name: request.name.clone(),
        owner: request.owner,
        co_owner: request.co_owner,
        locked: request.locked,
        code: None,
    })
    .unwrap();

    let kafka_topic = topics::team_info(&team.id);

    let mut active_model = team.into_active_model();
    active_model.name = set_option(request.name);
    active_model.owner = set_option(request.owner);
    active_model.co_owner = set_option(request.co_owner);
    active_model.locked = set_option(request.locked);

    teams::Entity::update(active_model).exec(&txn).await?;

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&kafka_topic)
                .partition(0)
                .payload(&kafka_payload),
            Duration::from_secs(5),
        )
        .await?;

    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

// This also checks if the user is actually exists, but does not differentiate
// between non-existing and not in team for security reasons
async fn is_user_in_team(
    db: &impl ConnectionTrait,
    user_id: &Uuid,
    team_id: &Uuid,
) -> Result<bool> {
    let team = teams::Entity::find_from_member(user_id).one(db).await?;

    if let Some(teams::Model { id, .. }) = team {
        Ok(*team_id == id)
    } else {
        Ok(false)
    }
}
