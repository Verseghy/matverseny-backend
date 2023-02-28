use crate::{
    error::{self, Result},
    json::Json,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{problems, problems_order};
use sea_orm::{EntityTrait, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    id: Uuid,
}

pub async fn delete_problem<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    // TODO: permission check through the iam

    let txn = state.db().begin().await?;

    let res = problems_order::Entity::find_by_id(request.id)
        .one(&txn)
        .await?;

    if res.is_some() {
        super::order::delete_problem(&txn, request.id).await?;
    }

    let res = problems::Entity::delete_by_id(request.id)
        .exec(&txn)
        .await?;

    if res.rows_affected == 0 {
        return Err(error::PROBLEM_NOT_FOUND);
    }

    // TODO: send kafka events

    txn.commit().await?;

    Ok(StatusCode::OK)
}
