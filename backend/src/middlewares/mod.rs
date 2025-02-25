mod claims;
mod permissions;

use crate::StateTrait;
use axum::{Router, http::header::AUTHORIZATION, middleware};
pub use permissions::*;
use std::iter;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    cors::{Any, CorsLayer},
};

pub fn middlewares<S: StateTrait>(state: S, router: Router<S>) -> Router {
    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let middlewares = ServiceBuilder::new()
        .catch_panic()
        .sensitive_headers(iter::once(AUTHORIZATION))
        .propagate_x_request_id()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            claims::get_claims::<S>,
        ))
        .compression()
        .decompression()
        .layer(cors_layer)
        .into_inner();

    router.layer(middlewares).with_state(state)
}
