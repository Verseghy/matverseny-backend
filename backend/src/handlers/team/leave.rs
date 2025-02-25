use crate::{
    Result, StateTrait, error, extractors::UserID, handlers::socket::Event, utils::topics,
};
use axum::{extract::State, http::StatusCode};
use entity::{team_members, teams, users};
use sea_orm::{EntityTrait, QuerySelect, TransactionTrait};

pub async fn leave_team<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let user = users::Entity::find_by_id(*user_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            warn!("tried to leave team without registration");
            error::USER_NOT_REGISTERED
        })?;

    let team = teams::Entity::find_from_member(&user.id)
        // NOTE: maybe not neccessary because locking the team (in the application and not in the database)
        //       while this handler is running shouldn't make invalid state in the database
        .lock_shared()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    if team.owner == user.id {
        return Err(error::OWNER_CANNOT_LEAVE);
    }

    team_members::Entity::delete_by_id((user.id, team.id))
        .exec(&txn)
        .await?;

    state
        .nats()
        .publish(
            topics::team_info(&team.id),
            serde_json::to_vec(&Event::LeaveTeam { user: user.id })
                .unwrap()
                .into(),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::OK)
}
