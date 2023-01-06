use crate::{
    error::{self, Result},
    handlers::socket::Event,
    iam::Claims,
    utils::topics,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{team_members, teams};
use rdkafka::producer::FutureRecord;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect, TransactionTrait};
use std::time::Duration;

pub async fn disband_team<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
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

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    team_members::Entity::delete_many()
        .filter(team_members::Column::TeamId.eq(team.id))
        .exec(&txn)
        .await?;

    teams::Entity::delete_by_id(team.id).exec(&txn).await?;

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&topics::team_info(&team.id))
                .partition(0)
                .payload(&serde_json::to_string(&Event::DisbandTeam).unwrap()),
            Duration::from_secs(5),
        )
        .await?;

    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
