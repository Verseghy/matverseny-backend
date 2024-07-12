use crate::{
    error::{self, DatabaseError as _},
    extractors::UserID,
    handlers::socket::Event,
    utils::topics,
    Json, Result, StateTrait,
};
use axum::{extract::State, http::StatusCode};
use entity::{
    team_members::{self, constraints::*},
    teams, users,
};
use sea_orm::{EntityTrait, QuerySelect, Set, TransactionTrait};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Request {
    code: String,
}

pub async fn join_team<S: StateTrait>(
    State(state): State<S>,
    user_id: UserID,
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

        let user = users::Entity::find_by_id(*user_id)
            .lock_exclusive()
            .one(&txn)
            .await?
            .ok_or_else(|| {
                // this is suspicious so log it
                warn!("tried to join team without registration");
                error::USER_NOT_REGISTERED
            })?;

        let active_model = team_members::ActiveModel {
            user_id: Set(user.id),
            team_id: Set(team.id),
        };

        let result = team_members::Entity::insert(active_model).exec(&txn).await;

        if let Err(err) = result {
            if err.unique_violation(UC_TEAM_MEMBERS_USER_ID) {
                return Err(error::ALREADY_IN_TEAM);
            }
        }

        let user_info = state
            .iam_app()
            .get_user_info(&format!("UserID-{}", &user.id))
            .await
            .map_err(|error| {
                error!("iam error: {:?}", error);
                error::IAM_FAILED_GET_NAME
            })?;

        state
            .nats()
            .publish(
                topics::team_info(&team.id),
                serde_json::to_vec(&Event::JoinTeam {
                    user: user.id,
                    name: user_info.name,
                })
                .unwrap()
                .into(),
            )
            .await?;

        txn.commit().await?;

        Ok(StatusCode::OK)
    } else {
        Err(error::JOIN_CODE_NOT_FOUND)
    }
}
