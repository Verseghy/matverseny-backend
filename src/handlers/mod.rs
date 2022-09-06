mod register;

use crate::shared::SharedTrait;
use axum::{routing::post, Router};

pub fn routes<S: SharedTrait>() -> Router {
    Router::new().route("/register", post(register::register::<S>))
}
