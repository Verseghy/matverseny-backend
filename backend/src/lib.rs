#[macro_use]
extern crate tracing;

pub mod error;
mod extractors;
mod handlers;
mod middlewares;
mod state;
mod utils;

use crate::{middlewares::middlewares, utils::SignalHandler};
use error::{Error, Result};
pub use state::*;
use tokio::net::TcpListener;
pub use utils::panic;

pub async fn run<S: StateTrait>(listener: TcpListener, state: S) -> anyhow::Result<()> {
    info!(
        "listening on port {}",
        listener.local_addr().unwrap().port()
    );

    let routes = handlers::routes::<S>(state.clone());
    let app = middlewares(state, routes);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(SignalHandler::new())
        .await?;

    Ok(())
}
