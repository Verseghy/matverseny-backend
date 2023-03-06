use crate::{
    error::{self, Result},
    json::Json,
    StateTrait,
};
use axum::{extract::State, http::StatusCode};
use chrono::{DateTime, NaiveDateTime, Utc};
use entity::times;
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Request {
    start_time: Option<i64>,
    end_time: Option<i64>,
}

pub async fn set_time_patch<S: StateTrait>(
    State(state): State<S>,
    Json(req): Json<Request>,
) -> Result<StatusCode> {
    let txn = state.db().begin().await?;

    if let Some(start_time) = req.start_time {
        let Some(naive) = NaiveDateTime::from_timestamp_opt(start_time, 0) else {
            error!("start_time seconds out of range!");
            return Err(error::TIME_SECONDS_OUT_OF_RANGE)
        };

        let time = DateTime::<Utc>::from_utc(naive, Utc);

        let model = times::ActiveModel {
            name: Set(times::constants::START_TIME.to_owned()),
            time: Set(time),
        };

        times::Entity::update(model).exec(&txn).await?;
    }

    if let Some(end_time) = req.end_time {
        let Some(naive) = NaiveDateTime::from_timestamp_opt(end_time, 0) else {
            error!("end_time seconds out of range!");
            return Err(error::TIME_SECONDS_OUT_OF_RANGE)
        };

        let time = DateTime::<Utc>::from_utc(naive, Utc);

        let model = times::ActiveModel {
            name: Set(times::constants::END_TIME.to_owned()),
            time: Set(time),
        };

        times::Entity::update(model).exec(&txn).await?;
    }

    txn.commit().await?;

    // TODO: send kafka events

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct PutRequest {
    start_time: i64,
    end_time: i64,
}

pub async fn set_time<S: StateTrait>(
    state: State<S>,
    Json(req): Json<PutRequest>,
) -> Result<StatusCode> {
    set_time_patch(
        state,
        Json(Request {
            start_time: Some(req.start_time),
            end_time: Some(req.end_time),
        }),
    )
    .await
}

#[derive(Debug, Serialize)]
pub struct GetResponse {
    start_time: i64,
    end_time: i64,
}

pub async fn get_time<S: StateTrait>(State(state): State<S>) -> Result<Json<GetResponse>> {
    let res = times::Entity::find()
        .filter(
            Condition::any()
                .add(times::Column::Name.eq("start_time"))
                .add(times::Column::Name.eq("end_time")),
        )
        .all(state.db())
        .await?;

    if res.len() != 2 {
        error!("start_time or end_time is not found in the database");
        return Err(error::INTERNAL);
    }

    let start_time = res.iter().find(|i| i.name == "start_time").unwrap();
    let end_time = res.iter().find(|i| i.name == "end_time").unwrap();

    Ok(Json(GetResponse {
        start_time: start_time.time.timestamp(),
        end_time: end_time.time.timestamp(),
    }))
}
