use crate::{
    error::{self, Result},
    handlers::socket::Event,
    utils::topics,
    StateTrait,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use entity::{problems, problems_order};
use sea_orm::{EntityTrait, TransactionTrait};
use uuid::Uuid;

pub async fn delete_problem<S: StateTrait>(
    State(state): State<S>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let res = problems_order::Entity::find_by_id(id).one(&txn).await?;

    if res.is_some() {
        super::order::delete_problem(&txn, id).await?;
    }

    let res = problems::Entity::delete_by_id(id).exec(&txn).await?;

    if res.rows_affected == 0 {
        return Err(error::PROBLEM_NOT_FOUND);
    }

    state
        .nats()
        .publish(
            topics::problems(),
            serde_json::to_vec(&Event::DeleteProblem { id })
                .unwrap()
                .into(),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::OK)
}
