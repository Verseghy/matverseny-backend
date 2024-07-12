#[macro_use]
extern crate tracing;

pub mod error;
mod extractors;
mod handlers;
mod json;
mod middlewares;
mod state;
mod utils;

use crate::{middlewares::middlewares, utils::SignalHandler};
use axum::{extract::Request, ServiceExt};
use error::{Error, Result};
use json::*;
pub use state::*;
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePath;
pub use utils::panic;

pub async fn run<S: StateTrait>(listener: TcpListener, state: S) -> anyhow::Result<()> {
    info!(
        "listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let app = handlers::routes::<S>(state.clone());
    let app = middlewares(state, app);
    let app = NormalizePath::trim_trailing_slash(app);

    Ok(
        axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
            .with_graceful_shutdown(SignalHandler::new())
            .await?,
    )
}
