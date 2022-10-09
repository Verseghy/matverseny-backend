use crate::{error, iam::Claims, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use sea_orm::{EntityTrait, FromQueryResult, IntoActiveModel, QuerySelect, Set, TransactionTrait};

#[derive(Debug, FromQueryResult)]
struct Team {
    locked: bool,
}

pub async fn leave_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = shared.db().begin().await?;

    let user = users::Entity::find_by_id(claims.subject)
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            tracing::warn!("tried to leave team without registration");
            error::USER_NOT_REGISTERED
        })?;

    if user.team.is_none() {}

    let team = users::Entity::select_team(&user.id)
        .select_only()
        .column(teams::Column::Locked)
        .into_model::<Team>()
        .one(&txn)
        .await?;

    if let Some(team) = team {
        if team.locked {
            return Err(error::LOCKED_TEAM);
        }

        let mut active_model = user.into_active_model();
        active_model.team = Set(None);

        users::Entity::update(active_model).exec(&txn).await?;

        txn.commit().await?;

        Ok(StatusCode::OK)
    } else {
        Err(error::USER_NOT_IN_TEAM)
    }
}
