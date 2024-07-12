use crate::{
    error::{self, Result},
    extractors::UserID,
    handlers::socket::Event,
    json::Json,
    utils::topics,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{team_members, teams, users};
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    user: Uuid,
}

pub async fn kick_user<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&user_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != *user_id && team.co_owner.as_ref() != Some(&user_id) {
        return Err(error::USER_NOT_COOWNER);
    }

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    if request.user == team.owner {
        return Err(error::CANNOT_KICK_OWNER);
    }

    if request.user == *user_id {
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

    let topic = topics::team_info(&team.id);

    if Some(request.user) == team.co_owner {
        let mut model = team.into_active_model();
        model.co_owner = Set(None);
        teams::Entity::update(model).exec(&txn).await?;
    }

    state
        .nats()
        .publish(
            topic,
            serde_json::to_vec(&Event::LeaveTeam { user: request.user })
                .unwrap()
                .into(),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
