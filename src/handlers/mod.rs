mod competition;
mod problem;
mod register;
pub mod socket;
mod stats;
mod team;

use crate::{middlewares::PermissionsLayer, state::StateTrait};
use axum::{
    handler::Handler,
    routing::{get, post},
    Router,
};

pub fn routes<S: StateTrait>(state: S) -> Router<S> {
    Router::new()
        .route("/register", post(register::register::<S>))
        .nest("/team", team::routes::<S>())
        .nest("/problem", problem::routes::<S>(state.clone()))
        .nest("/competition", competition::routes::<S>(state.clone()))
        .route("/ws", get(socket::ws_handler::<S>))
        .route(
            "/stats",
            get(stats::get_stats::<S>
                .layer(PermissionsLayer::new(state, &["mathcompetition.admin"]))),
        )
        .route("/liveness", get(|| async {}))
        .route("/readiness", get(|| async {}))
}
