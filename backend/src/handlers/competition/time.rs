use crate::{
    StateTrait,
    error::{self, Result},
    extractors::Json,
    handlers::socket::Event,
    utils::topics,
};
use axum::{extract::State, http::StatusCode};
use chrono::DateTime;
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

    let start_time = if let Some(start_time) = req.start_time {
        let Some(time) = DateTime::from_timestamp(start_time, 0) else {
            error!("start_time seconds out of range!");
            return Err(error::TIME_SECONDS_OUT_OF_RANGE);
        };

        let model = times::ActiveModel {
            name: Set(times::constants::START_TIME.to_owned()),
            time: Set(time),
        };

        times::Entity::update(model).exec(&txn).await?;

        Some(time)
    } else {
        None
    };

    let end_time = if let Some(end_time) = req.end_time {
        let Some(time) = DateTime::from_timestamp(end_time, 0) else {
            error!("end_time seconds out of range!");
            return Err(error::TIME_SECONDS_OUT_OF_RANGE);
        };

        let model = times::ActiveModel {
            name: Set(times::constants::END_TIME.to_owned()),
            time: Set(time),
        };

        times::Entity::update(model).exec(&txn).await?;

        Some(time)
    } else {
        None
    };

    state
        .nats()
        .publish(
            topics::times(),
            serde_json::to_vec(&Event::UpdateTime {
                start_time,
                end_time,
            })
            .unwrap()
            .into(),
        )
        .await?;

    txn.commit().await?;

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
