mod create;
mod delete;
mod list;
mod order;
mod update;

use crate::{middlewares::PermissionsLayer, StateTrait};
use axum::{
    routing::{delete, get, patch, post},
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
pub fn routes<S: StateTrait>(state: S) -> Router<S> {
    Router::new()
        .route(
            "/:id",
            get(list::get_problem::<S>)
                // .layer(PermissionsLayer::new(state, &["mathcompetition.problems"])),
        )
        .route("/", get(list::list_problems::<S>))
        .route("/", post(create::create_problem::<S>))
        .route("/", patch(update::update_problem::<S>))
        .route("/", delete(delete::delete_problem::<S>))
        .route("/order", post(order::change::<S>))
        .route("/order", get(order::get::<S>))
}
