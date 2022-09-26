pub mod error;
mod handlers;
mod iam;
mod json;
mod middlewares;
mod shared;
mod utils;

use error::{Error, Result};
use json::*;
pub use shared::*;

use axum::{http::header::AUTHORIZATION, Router};
use std::{iter::once, net::TcpListener};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    ServiceBuilderExt,
};

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    };
}

fn app<S: SharedTrait>(shared: S) -> Router {
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let middlewares = ServiceBuilder::new()
        .catch_panic()
        .sensitive_headers(once(AUTHORIZATION))
        .propagate_x_request_id()
        .add_extension(shared)
        .layer(middlewares::GetClaimsLayer::<S>::new())
        .compression()
        .decompression()
        .layer(cors_layer)
        .into_inner();

    handlers::routes::<S>().layer(middlewares)
}

pub async fn run(listener: TcpListener, shared: impl SharedTrait) {
    tracing::info!(
        "listening on port {}",
        listener.local_addr().unwrap().port()
    );

    axum::Server::from_tcp(listener)
        .expect("failed to start server")
        .serve(app(shared).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}
