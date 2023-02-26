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
    nonstandard_style
)]
#![deny(private_in_public)]
#![allow(elided_lifetimes_in_paths, clippy::type_complexity)]

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
