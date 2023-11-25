#[macro_use]
extern crate tracing;

pub mod error;
mod handlers;
mod iam;
mod json;
mod middlewares;
mod state;
mod utils;

use crate::{middlewares::middlewares, utils::SignalHandler};
use error::{Error, Result};
use json::*;
pub use state::*;
use std::net::TcpListener;
pub use utils::panic;

pub async fn run<S: StateTrait>(listener: TcpListener, state: S) {
    info!(
        "listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let app = handlers::routes::<S>(state.clone());
    let app = middlewares(state, app);

    axum::Server::from_tcp(listener)
        .expect("failed to start server")
        .serve(app.into_make_service())
        .with_graceful_shutdown(SignalHandler::new())
        .await
        .unwrap()
}
