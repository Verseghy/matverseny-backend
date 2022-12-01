use crate::{
    error, error::Error, handlers::socket::Event, iam::Claims, utils::topics, Result, StateTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{team_members, teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, QuerySelect, TransactionTrait};
use std::time::Duration;

pub async fn leave_team<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let user = users::Entity::find_by_id(claims.subject)
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
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&topics::team_info(&team.id))
                .partition(0)
                .payload(&serde_json::to_string(&Event::LeaveTeam { user: user.id }).unwrap()),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(err, _)| Error::internal(err))?;

    txn.commit().await?;

    Ok(StatusCode::OK)
}
