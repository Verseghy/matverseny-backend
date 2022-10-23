use crate::{
    error, error::Error, error::Result, handlers::socket::Event, iam::Claims, utils::set_option,
    SharedTrait, ValidatedJson,
};
use axum::{http::StatusCode, Extension};
use entity::{teams, users};
use rdkafka::producer::FutureRecord;
use sea_orm::{
    ConnectionTrait, EntityTrait, FromQueryResult, IntoActiveModel, QuerySelect, TransactionTrait,
};
use serde::Deserialize;
use std::time::Duration;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct Request {
    #[validate(length(max = 32))]
    name: Option<String>,
    owner: Option<String>,
    #[serde(default, with = "::serde_with::rust::double_option")]
    coowner: Option<Option<String>>,
    locked: Option<bool>,
}

pub async fn update_team<S: SharedTrait>(
    Extension(shared): Extension<S>,
    claims: Claims,
    ValidatedJson(request): ValidatedJson<Request>,
) -> Result<StatusCode> {
    let txn = shared.db().begin().await?;

    let team = users::Entity::select_team(&claims.subject)
        .lock_exclusive()
        .one(&txn)
        .await?;

    if let Some(team) = team {
        if team.owner != claims.subject {
            return Err(error::USER_NOT_OWNER);
        }

        // Allow updates when the team is locked, but the request sets it the be unlocked.
        // Otherwise error
        if (request.locked.is_none() || request.locked == Some(true)) && team.locked {
            return Err(error::LOCKED_TEAM);
        }

        // without this the ORM would generate an invalid sql statement
        if request.name.is_none()
            && request.owner.is_none()
            && request.coowner.is_none()
            && request.locked.is_none()
        {
            return Ok(StatusCode::NO_CONTENT);
        }

        if let Some(owner) = &request.owner {
            if owner == &claims.subject {
                return Ok(StatusCode::NO_CONTENT);
            }

            if !is_user_in_team(&txn, owner, &team.id).await? {
                return Err(error::NO_SUCH_MEMBER);
            }
        }

        if let Some(Some(coowner)) = &request.coowner {
            if !is_user_in_team(&txn, coowner, &team.id).await? {
                return Err(error::NO_SUCH_MEMBER);
            }
        }

        let kafka_topic = super::get_kafka_topic(&team.id);
        let kafka_payload = serde_json::to_string(&Event::UpdateTeam {
            name: request.name.clone(),
            owner: request.owner.clone(),
            coowner: request.coowner.clone(),
            locked: request.locked,
            code: None,
        })
        .unwrap();

        let mut active_model = team.into_active_model();
        active_model.name = set_option(request.name);
        active_model.owner = set_option(request.owner);
        active_model.coowner = set_option(request.coowner);
        active_model.locked = set_option(request.locked);

        teams::Entity::update(active_model).exec(&txn).await?;

        shared
            .kafka_producer()
            .send(
                FutureRecord::<(), String>::to(&kafka_topic)
                    .partition(0)
                    .payload(&kafka_payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(err, _)| Error::internal(err))?;

        txn.commit().await?;
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(error::USER_NOT_IN_TEAM)
    }
}

#[derive(FromQueryResult)]
struct User {
    team: Option<String>,
}

// This also checks if the user is actually exists, but does not differentiate
// between non-existing and not in team for security reasons
async fn is_user_in_team(db: &impl ConnectionTrait, user: &str, team: &str) -> Result<bool> {
    let res = users::Entity::find_by_id(user.to_string())
        .select_only()
        .column(users::Column::Team)
        .into_model::<User>()
        .one(db)
        .await?;

    Ok(if let Some(User { team: Some(t) }) = res {
        t == team
    } else {
        false
    })
}
