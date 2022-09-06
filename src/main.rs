use axum::{routing::get, Router, http::header::AUTHORIZATION};
mod shared;
mod json;
mod error;
use error::*;
use json::*;
use shared::*;
use std::{net::{Ipv4Addr, SocketAddr}, iter::once};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{cors::{Any, CorsLayer}, ServiceBuilderExt};
use tracing::level_filters::LevelFilter;

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

fn router() -> Router {
    Router::new().route("/", get(handler))
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
        .trace_for_http()
        .compression()
        .decompression()
        .layer(cors_layer)
        .into_inner();

    router().layer(middlewares)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .with_line_number(true)
        .compact()
        .init();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3002));
    let shared = Shared::new().await;

    tracing::info!("listening on port {}", addr.port());
    axum::Server::bind(&addr)
        .serve(app(shared).into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

async fn handler() -> String {
    String::from("hello world")
}
