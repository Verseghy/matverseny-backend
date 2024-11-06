use crate::{
    error::{self, Result},
    extractors::UserID,
    handlers::socket::Event,
    utils::topics,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{team_members, teams};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait};

pub async fn disband_team<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&user_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != *user_id {
        return Err(error::USER_NOT_OWNER);
    }

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    team_members::Entity::delete_many()
        .filter(team_members::Column::TeamId.eq(team.id))
        .exec(&txn)
        .await?;

    teams::Entity::delete_by_id(team.id).exec(&txn).await?;

    state
        .nats()
        .publish(
            topics::team_info(&team.id),
            serde_json::to_vec(&Event::DisbandTeam).unwrap().into(),
        )
        .await?;

    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
