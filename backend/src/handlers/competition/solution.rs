use crate::{
    StateTrait,
    error::{self, Result},
    extractors::{Json, UserID},
    handlers::socket::Event,
    utils::topics,
};
use axum::{extract::State, http::StatusCode};
use entity::{solutions_history, teams};
use sea_orm::{ActiveValue::Set, EntityTrait, QuerySelect, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Request {
    problem: Uuid,
    solution: Option<i64>,
}

pub async fn set_solution<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&user_id)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    let solution_history = solutions_history::ActiveModel {
        id: Set(Uuid::new_v4()),
        team: Set(team.id),
        problem: Set(request.problem),
        user: Set(*user_id),
        solution: Set(request.solution),
        created_at: Default::default(),
    };

    solutions_history::Entity::insert(solution_history)
        .exec(&txn)
        .await?;

    state
        .nats()
        .publish(
            topics::team_solutions(&team.id),
            serde_json::to_vec(&Event::SolutionSet {
                problem: request.problem,
                solution: request.solution,
            })
            .unwrap()
            .into(),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
