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
/// DELETE /problem/:id
///
/// POST   /problem/order
pub fn routes<S: StateTrait>(state: S) -> Router<S> {
    Router::new()
        .route(
            "/:id",
            get(list::get_problem::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/",
            get(list::list_problems::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/",
            post(create::create_problem::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/",
            patch(update::update_problem::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/:id",
            delete(delete::delete_problem::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/order",
            post(order::change::<S>).layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.problems"],
            )),
        )
        .route(
            "/order",
            get(order::get::<S>).layer(PermissionsLayer::new(state, &["mathcompetition.problems"])),
        )
}
