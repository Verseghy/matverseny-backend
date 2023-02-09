use crate::{
    error::{self, Result},
    json::Json,
    utils::set_option,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::problems;
use sea_orm::{DbErr, EntityTrait, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Request {
    id: Uuid,
    body: Option<String>,
    solution: Option<i64>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    image: Option<Option<String>>,
}

pub async fn update_problem<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    // TODO: permission check through the iam

    // This is necessary because the ORM would generate a wrong sql statement
    if request.body.is_none() && request.solution.is_none() && request.image.is_none() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let txn = state.db().begin().await?;

    let active_model = problems::ActiveModel {
        id: Set(request.id),
        body: set_option(request.body),
        solution: set_option(request.solution),
        image: set_option(request.image),
    };

    let res = problems::Entity::update(active_model).exec(&txn).await;

    match res {
        Err(DbErr::RecordNotFound(_)) => return Err(error::PROBLEM_NOT_FOUND),
        e => e?,
    };

    // TODO: send kafka events

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
