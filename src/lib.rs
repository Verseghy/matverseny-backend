#![warn(
    clippy::all,
    clippy::dbg_macro,
    clippy::todo,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::mem_forget,
    clippy::unused_self,
    clippy::filter_map_next,
    clippy::needless_continue,
    clippy::needless_borrow,
    clippy::match_wildcard_for_single_variants,
    clippy::if_let_mutex,
    clippy::await_holding_lock,
    clippy::match_on_vec_items,
    clippy::imprecise_flops,
    clippy::suboptimal_flops,
    clippy::lossy_float_literal,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::fn_params_excessive_bools,
    clippy::exit,
    clippy::inefficient_to_string,
    clippy::linkedlist,
    clippy::macro_use_imports,
    clippy::unnested_or_patterns,
    clippy::str_to_string,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
#![deny(private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]

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
