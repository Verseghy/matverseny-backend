use crate::{
    error::{self, Error, Result},
    handlers::socket::Event,
    iam::Claims,
    SharedTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set, TransactionTrait};
use std::time::Duration;

pub async fn disband_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = shared.db().begin().await?;

    let team = users::Entity::select_team(&claims.subject)
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            tracing::warn!("tried to disband a team without registration");
            error::USER_NOT_REGISTERED
        })?;

    if team.owner != claims.subject {
        return Err(error::USER_NOT_OWNER);
    }

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    let users = users::Entity::find()
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

    shared
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
