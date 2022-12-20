mod problem;
mod register;
mod socket;
mod team;

use crate::state::StateTrait;
use axum::{
    routing::{get, post},
    Router,
};

pub fn routes<S: StateTrait>() -> Router<S> {
    Router::new()
        .route("/register", post(register::register::<S>))
        .nest("/team", team::routes::<S>())
        .nest("/problem", problem::routes::<S>())
        .route("/ws", get(socket::ws_handler::<S>))
}
