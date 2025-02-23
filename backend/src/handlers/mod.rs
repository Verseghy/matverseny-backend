mod competition;
mod problem;
mod register;
pub mod socket;
mod stats;
mod team;

use crate::{middlewares::PermissionsLayer, state::StateTrait};
use axum::{
    Router,
    extract::State,
    handler::Handler,
    http::StatusCode,
    routing::{get, post},
};
use sea_orm::ConnectionTrait;

pub fn routes<S: StateTrait>(state: S) -> Router<S> {
    Router::new()
        .nest(
            "/v1",
            Router::new()
                .route("/register", post(register::register::<S>))
                .nest("/team", team::routes::<S>(state.clone()))
                .nest("/problem", problem::routes::<S>(state.clone()))
                .nest("/competition", competition::routes::<S>(state.clone()))
                .route("/ws", get(socket::ws_handler::<S>))
                .route(
                    "/stats",
                    post(
                        stats::get_stats::<S>
                            .layer(PermissionsLayer::new(state, &["mathcompetition.admin"])),
                    ),
                ),
        )
        .route("/livez", get(liveness::<S>))
        .route("/readyz", get(|| async {}))
}

async fn liveness<S: StateTrait>(State(state): State<S>) -> StatusCode {
    if state.db().execute_unprepared("select 1").await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if state.nats().connection_state() != async_nats::connection::State::Connected {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
