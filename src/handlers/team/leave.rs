use crate::{error, iam::Claims, Result, SharedTrait};
use axum::{http::StatusCode, Extension};
use entity::users;
use sea_orm::{EntityTrait, IntoActiveModel, Set};

pub async fn leave_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let user = users::Entity::find_by_id(claims.subject)
        .one(shared.db())
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            tracing::warn!("tried to leave team without registration");
            error::USER_NOT_REGISTERED
        })?;

    if user.team.is_none() {
        return Err(error::USER_NOT_IN_TEAM);
    }

    let mut active_model = user.into_active_model();
    active_model.team = Set(None);

    users::Entity::update(active_model)
        .exec(shared.db())
        .await?;

    Ok(StatusCode::OK)
}
