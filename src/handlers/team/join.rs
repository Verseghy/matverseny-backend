use crate::{
    error::{self, DatabaseError as _},
    handlers::socket::Event,
    iam::Claims,
    utils::topics,
    Error, Json, Result, StateTrait,
};
use axum::{http::StatusCode, Extension};
use entity::{team_members, teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{EntityTrait, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct Request {
    code: String,
}

pub async fn join_team<S: StateTrait>(
    Extension(state): Extension<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    let team = teams::Entity::find_by_join_code(&request.code)
        // NOTE: maybe not neccessary because locking the team (in the application and not in the database)
        //       while this handler is running shouldn't make invalid state in the database
        .lock_shared()
        .one(&txn)
        .await?;

    if let Some(team) = team {
        if team.locked {
            return Err(error::LOCKED_TEAM);
        }

        let user = users::Entity::find_by_id(claims.subject)
            .lock_exclusive()
            .one(&txn)
            .await?
            .ok_or_else(|| {
                // this is suspicious so log it
                tracing::warn!("tried to join team without registration");
                error::USER_NOT_REGISTERED
            })?;

        let active_model = team_members::ActiveModel {
            user_id: Set(user.id),
            team_id: Set(team.id),
        };

        let result = team_members::Entity::insert(active_model).exec(&txn).await;

        if let Err(err) = result {
            if err.unique_violation("UC_team_members_user_id") {
                return Err(error::ALREADY_IN_TEAM);
            }
        }

        state
            .kafka_producer()
            .send(
                FutureRecord::<(), String>::to(&topics::team_info(&team.id))
                    .partition(0)
                    .payload(&serde_json::to_string(&Event::JoinTeam { user: user.id }).unwrap()),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| Error::internal(err))?;

        txn.commit().await?;

        Ok(StatusCode::OK)
    } else {
        Err(error::JOIN_CODE_NOT_FOUND)
    }
}
