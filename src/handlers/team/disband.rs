use crate::{
    error::{self, Error, Result},
    handlers::socket::Event,
    iam::Claims,
    StateTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{
    ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect, Set, TransactionTrait,
};
use std::time::Duration;

pub async fn disband_team<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = users::Entity::select_team(&claims.subject)
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

    let users = users::Entity::find()
        .lock_exclusive()
        .filter(users::Column::Team.eq(&*team.id))
        .all(&txn)
        .await?;

    for user in users {
        let mut model = user.into_active_model();
        model.team = Set(None);

        users::Entity::update(model).exec(&txn).await?;
    }

    teams::Entity::delete_by_id(team.id.clone())
        .exec(&txn)
        .await?;

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&super::get_kafka_topic(&team.id))
                .partition(0)
                .payload(&serde_json::to_string(&Event::DisbandTeam).unwrap()),
            Duration::from_secs(5),
        )
        .await
        .map_err(|(err, _)| Error::internal(err))?;

    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
