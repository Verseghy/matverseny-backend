use crate::{error::Result, iam::Claims, json::Json, StateTrait};
use axum::{extract::State, http::StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Request {
    problem: String,
    solution: i64,
}

pub async fn set_solution<S: StateTrait>(
    State(state): State<S>,
    claims: Claims,
    Json(request): Json<Request>,
) -> Result<StatusCode> {
    todo!()
}
