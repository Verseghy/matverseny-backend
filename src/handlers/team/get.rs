use crate::{error::Result, json::Json, StateTrait};
use axum::extract::State;
use entity::{teams, users};
use sea_orm::{EntityTrait, FromQueryResult, TransactionTrait};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, FromQueryResult)]
pub struct Member {
    id: Uuid,
    school: String,
    class: i16,
}

#[derive(Debug, Serialize)]
pub struct Team {
    id: Uuid,
    name: String,
    owner: Uuid,
    co_owner: Option<Uuid>,
    locked: bool,
    join_code: String,
    members: Vec<Member>,
}

pub type Response = Json<Vec<Team>>;

pub async fn get_all_teams<S: StateTrait>(State(state): State<S>) -> Result<Response> {
    let txn = state.db().begin().await?;

    let teams = teams::Entity::find().all(&txn).await?;

    let mut response = Vec::with_capacity(teams.len());

    for team in teams {
        let members = users::Entity::find_in_team(&team.id)
            .into_model::<Member>()
            .all(&txn)
            .await?;

        response.push(Team {
            id: team.id,
            name: team.name,
            owner: team.owner,
            co_owner: team.co_owner,
            locked: team.locked,
            join_code: team.join_code,
            members,
        })
    }

    txn.commit().await?;
    Ok(Json(response))
}
