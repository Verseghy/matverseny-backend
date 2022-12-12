mod create;

use crate::StateTrait;
use axum::{routing::post, Router};

/// Routes for problem management
///
/// GET    /problem
/// GET    /problem/:id
/// POST   /problem
/// PATCH  /problem
/// DELETE /problem
///
/// POST   /problem/order
pub fn routes<S: StateTrait>() -> Router {
    Router::new().route("/", post(create::create_problem::<S>))
}
