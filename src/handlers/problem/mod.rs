mod create;
mod delete;
mod list;

use crate::StateTrait;
use axum::{
    routing::{delete, get, post},
    Router,
};

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
    Router::new()
        .route("/:id", get(list::get_problem::<S>))
        .route("/", get(list::list_problems::<S>))
        .route("/", post(create::create_problem::<S>))
        .route("/", delete(delete::delete_problem::<S>))
}
