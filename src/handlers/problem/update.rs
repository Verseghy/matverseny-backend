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
        Err(DbErr::RecordNotUpdated) => return Err(error::PROBLEM_NOT_FOUND),
        e => e?,
    };

    // TODO: send kafka events

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct PutRequest {
    id: Uuid,
    body: String,
    solution: i64,
    image: Option<String>,
}

pub async fn put<S: StateTrait>(
    state: State<S>,
    Json(request): Json<PutRequest>,
) -> Result<StatusCode> {
    update_problem(
        state,
        Json(Request {
            id: request.id,
            body: Some(request.body),
            solution: Some(request.solution),
            image: Some(request.image),
        }),
    )
    .await
}
