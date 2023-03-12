use std::time::Duration;
use crate::{error, error::Result, iam::Claims, json::Json, StateTrait};
use axum::{extract::State, http::StatusCode};
use sea_orm::ActiveValue::Set;
use serde::Deserialize;
use uuid::Uuid;
use entity::{solutions_history, teams};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, QuerySelect, TransactionTrait};
use crate::handlers::socket::Event;
use crate::utils::topics;

#[derive(Debug, Deserialize)]
pub struct Request {
    problem: Uuid,
    solution: Option<i64>,
}

pub async fn set_solution<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    let solution_history = solutions_history::ActiveModel{
        id: Set(Uuid::new_v4()),
        team: Set(team.id),
        problem: Set(request.problem),
        user: Set(claims.subject),
        solution: Set(request.solution),
        created_at: Default::default(),
    };

    solutions_history::Entity::insert(solution_history).exec(state.db()).await?;

    state
        .kafka_producer()
        .send(
            FutureRecord::<(), String>::to(&topics::team_solutions(&team.id))
                .partition(0)
                .payload(
                    &serde_json::to_string(&Event::SolutionSet {
                        problem: request.problem,
                        solution: request.solution
                    })
                        .unwrap(),
                ),
            Duration::from_secs(5),
        )
        .await?;

    txn.commit().await?;

    Ok(StatusCode::NO_CONTENT)
}
