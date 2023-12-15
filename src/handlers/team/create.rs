use crate::{
    error::{self, DatabaseError, Result},
    iam::Claims,
    utils::{generate_join_code, topics},
    StateTrait, ValidatedJson,
};
use axum::{extract::State, http::StatusCode};
use entity::{
    team_members::{self, constraints::*},
    teams::{self, constrains::*},
    users,
};
use rdkafka::admin::{AdminOptions, NewTopic, TopicReplication};
use sea_orm::{EntityTrait, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct Request {
    #[validate(length(max = 32))]
    name: String,
}

pub async fn create_team<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
    ValidatedJson(request): ValidatedJson<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let user = users::Entity::find_by_id(claims.subject)
        .lock_shared()
        .one(&txn)
        .await?
        .ok_or_else(|| {
            // this is suspicious so log it
            warn!("tried to create team without registration");
            error::USER_NOT_REGISTERED
        })?;

    let team = teams::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(request.name),
        owner: Set(user.id),
        locked: Set(false),
        ..Default::default()
    };

    for _ in 0..16 {
        let team_model = {
            let mut model = team.clone();
            model.join_code = Set(generate_join_code(&mut state.rng()));
            model
        };

        let result = match teams::Entity::insert(team_model).exec(&txn).await {
            Err(err) if err.unique_violation(UC_TEAMS_NAME) => {
                return Err(error::DUPLICATE_TEAM_NAME)
            }
            Err(err) if err.unique_violation(UC_TEAMS_JOIN_CODE) => continue,
            r => r?,
        };

        let team_member_model = team_members::ActiveModel {
            user_id: Set(user.id),
            team_id: Set(result.last_insert_id),
        };

        match team_members::Entity::insert(team_member_model)
            .exec(&txn)
            .await
        {
            Err(err) if err.unique_violation(UC_TEAM_MEMBERS_USER_ID) => {
                return Err(error::ALREADY_IN_TEAM)
            }
            r => r?,
        };

        state
            .kafka_admin()
            .create_topics(
                &[
                    NewTopic::new(
                        &topics::team_info(&result.last_insert_id),
                        1,
                        TopicReplication::Fixed(1),
                    ),
                    NewTopic::new(
                        &topics::team_solutions(&result.last_insert_id),
                        1,
                        TopicReplication::Fixed(1),
                    ),
                ],
                &AdminOptions::new(),
            )
            .await?;

        txn.commit().await?;

        return Ok(StatusCode::CREATED);
    }

    Err(error::FAILED_TO_GENERATE_JOIN_CODE)
}
