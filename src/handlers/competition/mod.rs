mod time;

use crate::{middlewares::PermissionsLayer, StateTrait};
use axum::{
    handler::Handler,
    routing::{get, patch, put},
    Router,
};

/// Routes for competition
///
/// POST /competition/solution
///
/// # Admin actions (users will get time on the socket)
/// PUT   /competition/time
/// PATCH /competition/time
/// GET   /competition/time
pub fn routes<S: StateTrait>(state: S) -> Router<S> {
    Router::new()
        .route(
            "/time",
            put(time::set_time::<S>.layer(PermissionsLayer::new(
                state.clone(),
                &["mathcompetition.admin"],
            ))),
        )
        .route(
            "/time",
            patch(
                time::set_time_patch::<S>
                    .layer(PermissionsLayer::new(state, &["mathcompetition.admin"])),
            ),
        )
        .route("/time", get(time::get_time::<S>))
}
