use crate::{error::Result, json::Json, StateTrait};
use axum::extract::State;
use entity::{problems, solutions_history, teams, users};
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, FromQueryResult)]
struct Member {
    id: Uuid,
    school: String,
    class: i16,
}

#[derive(Debug, Serialize)]
struct Answer {
    problem: Uuid,
    answer: Option<i64>,
}

#[derive(Debug, Serialize)]
struct Team {
    id: Uuid,
    name: String,
    members: Vec<Member>,
    answers: Vec<Answer>,
}

#[derive(Debug, Serialize, FromQueryResult)]
struct Solution {
    id: Uuid,
    solution: i64,
}

#[derive(Debug, Serialize)]
pub struct Response {
    teams: Vec<Team>,
    solutions: Vec<Solution>,
}

pub async fn get_stats<S: StateTrait>(State(state): State<S>) -> Result<Json<Response>> {
    let solutions = problems::Entity::find()
        .select_only()
        .column(problems::Column::Id)
        .column(problems::Column::Solution)
        .into_model::<Solution>()
        .all(state.db())
        .await?;

    let teams = teams::Entity::find().all(state.db()).await?;

    let mut final_teams = Vec::with_capacity(teams.len());

    for team in teams {
        let members = users::Entity::find_in_team(&team.id)
            .into_model::<Member>()
            .all(state.db())
            .await?;

        let answers = solutions_history::Entity::find()
            .filter(solutions_history::Column::Team.eq(team.id))
            .distinct_on([solutions_history::Column::Problem])
            .order_by_desc(solutions_history::Column::Problem)
            .order_by_desc(solutions_history::Column::CreatedAt)
            .all(state.db())
            .await?;

        final_teams.push(Team {
            id: team.id,
            name: team.name,
            members,
            answers: answers
                .into_iter()
                .map(|x| Answer {
                    problem: x.problem,
                    answer: x.solution,
                })
                .collect(),
        })
    }

    Ok(Json(Response {
        teams: final_teams,
        solutions,
    }))
}
