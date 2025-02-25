use crate::{
    StateTrait,
    error::{self, DatabaseError, Result},
    extractors::Json,
    handlers::socket::Event,
    utils::{execute_str, sort_linked, topics},
};
use axum::{extract::State, http::StatusCode};
use const_format::formatcp;
use entity::{
    problems,
    problems_order::{self, constraints::*},
};
use sea_orm::{
    ActiveValue::NotSet,
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect, Set,
    StatementBuilder, TransactionTrait,
    sea_query::{CaseStatement, Query},
};
use serde::{Deserialize, Serialize};
use smallvec::{SmallVec, smallvec};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Request {
    Insert { before: Option<Uuid>, id: Uuid },
    Delete { id: Uuid },
    Swap { id1: Uuid, id2: Uuid },
}

pub async fn change<S: StateTrait>(
    State(state): State<S>,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    match request {
        Request::Insert { before, id } => {
            if let Some(before) = before {
                let after = problems_order::Entity::find()
                    .filter(problems_order::Column::Next.eq(before))
                    .lock_exclusive()
                    .one(&txn)
                    .await?;

                execute_str(
                    &txn,
                    formatcp!(r#"SET CONSTRAINTS "{UC_PROBLEMS_ORDER_NEXT}" DEFERRED"#),
                )
                .await?;

                let res = problems_order::Entity::insert(problems_order::ActiveModel {
                    id: Set(id),
                    next: Set(Some(before)),
                })
                .exec(&txn)
                .await;

                match res {
                    Err(err) if err.foreign_key_violation(FK_PROBLEMS_ORDER_ID) => {
                        return Err(error::PROBLEM_NOT_FOUND);
                    }
                    Err(err) if err.unique_violation(PK_PROBLEMS_ORDER) => {
                        return Err(error::PROBLEM_ALREADY_IN_ORDER);
                    }
                    Err(err) => return Err(err.into()),
                    _ => {}
                }

                if let Some(after) = &after {
                    problems_order::Entity::update(problems_order::ActiveModel {
                        id: Set(after.id),
                        next: Set(Some(id)),
                    })
                    .exec(&txn)
                    .await?;
                }

                let res = problems::Entity::find_by_id(id).one(&txn).await?.unwrap();

                state
                    .nats()
                    .publish(
                        topics::problems(),
                        serde_json::to_vec(&Event::InsertProblem {
                            before: Some(before),
                            id: res.id,
                            body: res.body,
                            image: res.image,
                        })
                        .unwrap()
                        .into(),
                    )
                    .await?;

                txn.commit().await?;
            } else {
                // Insert problem to the end of the list

                let before = problems_order::Entity::find()
                    .filter(problems_order::Column::Next.is_null())
                    .lock_exclusive()
                    .one(&txn)
                    .await?;

                execute_str(
                    &txn,
                    formatcp!(r#"SET CONSTRAINTS "{UC_PROBLEMS_ORDER_NEXT}" DEFERRED"#),
                )
                .await?;

                let res = problems_order::Entity::insert(problems_order::ActiveModel {
                    id: Set(id),
                    next: NotSet,
                })
                .exec(&txn)
                .await;

                match res {
                    Err(err) if err.foreign_key_violation(FK_PROBLEMS_ORDER_ID) => {
                        return Err(error::PROBLEM_NOT_FOUND);
                    }
                    Err(err) if err.unique_violation(PK_PROBLEMS_ORDER) => {
                        return Err(error::PROBLEM_ALREADY_IN_ORDER);
                    }
                    Err(err) => return Err(err.into()),
                    _ => {}
                }

                if let Some(before) = before {
                    problems_order::Entity::update(problems_order::ActiveModel {
                        id: Set(before.id),
                        next: Set(Some(id)),
                    })
                    .exec(&txn)
                    .await?;
                }

                let res = problems::Entity::find_by_id(id).one(&txn).await?.unwrap();

                state
                    .nats()
                    .publish(
                        topics::problems(),
                        serde_json::to_vec(&Event::InsertProblem {
                            before: None,
                            id: res.id,
                            body: res.body,
                            image: res.image,
                        })
                        .unwrap()
                        .into(),
                    )
                    .await?;

                txn.commit().await?;
            }
        }
        Request::Delete { id } => {
            delete_problem(&txn, id).await?;

            state
                .nats()
                .publish(
                    topics::problems(),
                    serde_json::to_vec(&Event::DeleteProblem { id })
                        .unwrap()
                        .into(),
                )
                .await?;

            txn.commit().await?;
        }
        Request::Swap { id1, id2 } => {
            trace!("swapping: {id1}, {id2}");

            let res = problems_order::Entity::find()
                .filter(
                    Condition::any()
                        .add(problems_order::Column::Id.eq(id1))
                        .add(problems_order::Column::Id.eq(id2))
                        .add(problems_order::Column::Next.eq(id1))
                        .add(problems_order::Column::Next.eq(id2)),
                )
                .lock_exclusive()
                .all(&txn)
                .await?;

            let item1 = res.iter().find(|item| item.id == id1);
            let item2 = res.iter().find(|item| item.id == id2);
            let before1 = res.iter().find(|item| item.next == Some(id1));
            let before2 = res.iter().find(|item| item.next == Some(id2));

            let (Some(item1), Some(item2)) = (item1, item2) else {
                return Err(error::PROBLEM_NOT_FOUND);
            };

            let (expr, ids): (_, SmallVec<[Uuid; 4]>) =
                if item1.next == Some(item2.id) || item2.next == Some(item1.id) {
                    trace!("swap adjacent");

                    let (item1, item2, before2) = if item1.next == Some(item2.id) {
                        (item2, item1, before1)
                    } else {
                        (item1, item2, before2)
                    };

                    let mut ids = smallvec![item1.id, item2.id];

                    let mut expr = CaseStatement::new()
                        .case(problems_order::Column::Id.eq(item1.id), item2.id)
                        .case(problems_order::Column::Id.eq(item2.id), item1.next);

                    if let Some(before2) = before2 {
                        expr = expr.case(problems_order::Column::Id.eq(before2.id), item1.id);
                        ids.push(before2.id);
                    }

                    (expr, ids)
                } else {
                    trace!("swap not adjacent");

                    let mut ids = smallvec![item1.id, item2.id];

                    let mut expr = CaseStatement::new()
                        .case(problems_order::Column::Id.eq(item1.id), item2.next)
                        .case(problems_order::Column::Id.eq(item2.id), item1.next);

                    if let Some(before1) = before1 {
                        expr = expr.case(problems_order::Column::Id.eq(before1.id), item2.id);

                        ids.push(before1.id);
                    }

                    if let Some(before2) = before2 {
                        expr = expr.case(problems_order::Column::Id.eq(before2.id), item1.id);

                        ids.push(before2.id);
                    }

                    (expr, ids)
                };

            execute_str(
                &txn,
                formatcp!(r#"SET CONSTRAINTS "{UC_PROBLEMS_ORDER_NEXT}" DEFERRED"#),
            )
            .await?;

            let query = Query::update()
                .table(problems_order::Entity)
                .value(problems_order::Column::Next, expr)
                .and_where(problems_order::Column::Id.is_in(ids))
                .to_owned();

            txn.execute(StatementBuilder::build(&query, &txn.get_database_backend()))
                .await?;

            state
                .nats()
                .publish(
                    topics::problems(),
                    serde_json::to_vec(&Event::SwapProblems { id1, id2 })
                        .unwrap()
                        .into(),
                )
                .await?;

            txn.commit().await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get<S: StateTrait>(State(state): State<S>) -> Result<Json<Vec<Uuid>>> {
    let res = problems_order::Entity::find().all(state.db()).await?;

    debug!("res: {res:?}");

    if res.is_empty() {
        return Ok(Json(Vec::new()));
    }

    let sorted = sort_linked(res);
    let ids = sorted.into_iter().map(|item| item.id).collect();

    Ok(Json(ids))
}

pub(super) async fn delete_problem<T>(txn: &T, id: Uuid) -> Result<()>
where
    T: TransactionTrait + ConnectionTrait,
{
    let res = problems_order::Entity::find()
        .filter(
            Condition::any()
                .add(problems_order::Column::Id.eq(id))
                .add(problems_order::Column::Next.eq(id)),
        )
        .lock_exclusive()
        .all(txn)
        .await?;

    let to_delete = res.iter().find(|item| item.id == id);
    let before = res.iter().find(|item| item.next == Some(id));

    let Some(to_delete) = to_delete else {
        return Err(error::PROBLEM_NOT_FOUND);
    };

    execute_str(
        txn,
        formatcp!(r#"SET CONSTRAINTS "{FK_PROBLEMS_ORDER_NEXT}" DEFERRED"#),
    )
    .await?;

    problems_order::Entity::delete_by_id(to_delete.id)
        .exec(txn)
        .await?;

    if let Some(before) = before {
        problems_order::Entity::update(problems_order::ActiveModel {
            id: Set(before.id),
            next: Set(to_delete.next),
        })
        .exec(txn)
        .await?;
    }

    Ok(())
}
