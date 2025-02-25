use crate::StateTrait;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use headers::{Authorization, HeaderMapExt, authorization::Bearer};
use tracing::Instrument;

pub async fn get_claims<S: StateTrait>(
    State(state): State<S>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(header) = request.headers().typed_get::<Authorization<Bearer>>() else {
        return next.run(request).await;
    };

    let Ok(claims) = state.jwt().get_claims(header.token()).await else {
        return next.run(request).await;
    };

    let span = info_span!("claims", user_id = claims.sub.to_string());

    request.extensions_mut().insert(claims);

    next.run(request).instrument(span).await
}
