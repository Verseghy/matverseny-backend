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
use axum::ServiceExt;
use error::{Error, Result};
use json::*;
pub use state::*;
use std::net::TcpListener;
use tower_http::normalize_path::NormalizePath;
pub use utils::panic;

pub async fn run<S: StateTrait>(listener: TcpListener, state: S) {
    info!(
        "listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let app = handlers::routes::<S>(state.clone());
    let app = middlewares(state, app);
    let app = NormalizePath::trim_trailing_slash(app);

    axum::Server::from_tcp(listener)
        .expect("failed to start server")
        .serve(app.into_make_service())
        .with_graceful_shutdown(SignalHandler::new())
        .await
        .unwrap()
}
