use crate::{error::Result, extractors::Json, StateTrait};
use axum::extract::State;
use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, FromQueryResult, Statement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Request {
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TeamData {
    team_id: Uuid,
    correct: i64,
    wrong: i64,
}

pub type Response = Vec<TeamData>;

pub async fn get_stats<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<Json<Response>> {
    static SQL: &str = "
        select 
          id, 
          bools, 
          coalesce(count, 0) as count 
        from 
          teams 
          cross join (
            values 
              (true), 
              (false)
          ) as bools(bools)
          full outer join (
            select 
              team, 
              correct, 
              count(*) as count 
            from 
              (
                select 
                  distinct on (team, problem) team, 
                  solutions_history.solution = problems.solution as correct 
                from 
                  solutions_history 
                  inner join problems on problems.id = solutions_history.problem 
                where 
                  created_at < $1
                order by 
                  team, 
                  problem, 
                  created_at desc
              ) 
            where
              correct is not null
            group by 
              team, 
              correct
          ) as correct on correct.team = teams.id 
          and correct.correct = bools
        order by
          id,
          bools;
    ";

    let db = state.db();

    let res = db
        .query_all(Statement {
            sql: SQL.to_owned(),
            values: Some(sea_orm::Values(vec![request.timestamp.into()])),
            db_backend: db.get_database_backend(),
        })
        .await?;

    let mut map = BTreeMap::<Uuid, TeamData>::new();

    for row in &res {
        #[derive(FromQueryResult)]
        struct Row {
            id: Uuid,
            bools: bool,
            count: i64,
        }

        let row = Row::from_query_result(row, "")?;

        let slot = match map.get_mut(&row.id) {
            Some(s) => s,
            None => {
                map.insert(
                    row.id,
                    TeamData {
                        team_id: row.id,
                        correct: 0,
                        wrong: 0,
                    },
                );
                map.get_mut(&row.id).unwrap()
            }
        };

        if row.bools {
            slot.correct = row.count;
        } else {
            slot.wrong = row.count;
        }
    }

    // let query = Query::select()
    //     .column(teams::Column::Id)
    //     .column(Alias::new("bools"))
    //     .expr_as(
    //         Func::coalesce([Expr::col(Alias::new("count")).into(), Expr::val(0).into()]),
    //         Alias::new("count"),
    //     )
    //     .from(teams::Entity)
    //     .join(
    //         JoinType::CrossJoin,
    //         TableRef::ValuesList(
    //             vec![
    //                 ValueTuple::One(Value::Bool(Some(true))),
    //                 ValueTuple::One(Value::Bool(Some(false))),
    //             ],
    //             Alias::new("bools(bools)").into_iden(),
    //         ),
    //         Cond::any(),
    //     )
    //     .join(
    //         JoinType::FullOuterJoin,
    //         TableRef::SubQuery(
    //             Query::select()
    //                 .column(solutions_history::Column::Team)
    //                 .column((Alias::new("sub1"), Alias::new("correct")))
    //                 .expr_as(Func::count(Expr::col(Asterisk)), Alias::new("count"))
    //                 .from_subquery(
    //                     Query::select()
    //                         .distinct_on([
    //                             solutions_history::Column::Team,
    //                             solutions_history::Column::Problem,
    //                         ])
    //                         .column(solutions_history::Column::Team)
    //                         .expr_as(
    //                             Expr::col(solutions_history::Column::Solution)
    //                                 .equals(problems::Column::Solution),
    //                             Alias::new("correct"),
    //                         )
    //                         .from(solutions_history::Entity)
    //                         .join(
    //                             JoinType::InnerJoin,
    //                             problems::Entity,
    //                             Expr::col(problems::Column::Id)
    //                                 .eq(Expr::col(solutions_history::Column::Problem)),
    //                         )
    //                         .and_where(Expr::col(solutions_history::Column::Solution).is_not_null())
    //                         .and_where(
    //                             Expr::col(solutions_history::Column::CreatedAt)
    //                                 .lt(request.timestamp),
    //                         )
    //                         .order_by_columns([
    //                             (solutions_history::Column::Team, Order::Desc),
    //                             (solutions_history::Column::Problem, Order::Desc),
    //                             (solutions_history::Column::CreatedAt, Order::Desc),
    //                         ])
    //                         .take(),
    //                     Alias::new("sub1"),
    //                 )
    //                 .group_by_col(solutions_history::Column::Team)
    //                 .group_by_col((Alias::new("sub1"), Alias::new("correct")))
    //                 .to_owned(),
    //             Alias::new("correct").into_iden(),
    //         ),
    //         Cond::all()
    //             .add(
    //                 Expr::col((Alias::new("correct"), Alias::new("team")))
    //                     .eq(Expr::col(teams::Column::Id)),
    //             )
    //             .add(
    //                 Expr::col((Alias::new("correct"), Alias::new("correct")))
    //                     .eq(Expr::col(Alias::new("bools"))),
    //             ),
    //     )
    //     .and_where(teams::Column::Locked.eq(true))
    //     .to_owned();

    let teams = map.into_values().collect();

    Ok(Json(teams))
}
