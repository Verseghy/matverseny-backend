use crate::{
    error::{self, DatabaseError, Result},
    handlers::socket::Event,
    iam::Claims,
    utils::{self, topics},
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::teams::{self, constrains::*};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, IntoActiveModel, QuerySelect, Set, TransactionTrait};
use std::time::Duration;

pub async fn regenerate_code<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_from_member(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?
        .ok_or(error::USER_NOT_IN_TEAM)?;

    if team.owner != claims.subject && team.co_owner != Some(claims.subject) {
        return Err(error::USER_NOT_COOWNER);
    }

    if team.locked {
        return Err(error::LOCKED_TEAM);
    }

    let kafka_topic = topics::team_info(&team.id);
    let model = team.into_active_model();

    for _ in 0..16 {
        let new_code = utils::generate_join_code(&mut state.rng());

        let mut model = model.clone();
        model.join_code = Set(new_code.clone());

        let res = teams::Entity::update(model).exec(&txn).await;

        match res {
            Err(err) if err.unique_violation(UC_TEAMS_JOIN_CODE) => continue,
            r => r?,
        };

        let kafka_payload = serde_json::to_string(&Event::UpdateTeam {
            name: None,
            owner: None,
            co_owner: None,
            locked: None,
            code: Some(new_code),
        })
        .unwrap();

        state
            .kafka_producer()
            .send(
                FutureRecord::<(), String>::to(&kafka_topic)
                    .partition(0)
                    .payload(&kafka_payload),
                Duration::from_secs(5),
            )
            .await?;

        txn.commit().await?;

        return Ok(StatusCode::NO_CONTENT);
    }

    Err(error::FAILED_TO_GENERATE_JOIN_CODE)
}
