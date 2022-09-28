mod register;
mod socket;
mod team;

use crate::shared::SharedTrait;
use axum::{
    routing::{get, post},
    Router,
};

pub fn routes<S: SharedTrait>() -> Router {
    Router::new()
        .route("/register", post(register::register::<S>))
        .nest("/team", team::routes::<S>())
        .route("/ws", get(socket::ws_handler::<S>))
}
