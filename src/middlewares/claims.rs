use crate::{iam::IamTrait, StateTrait};
use axum::{
    headers::{
        authorization::{Authorization, Bearer},
        HeaderMapExt,
    },
    http::Request,
};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Span;

#[derive(Debug, Clone)]
pub struct GetClaimsLayer<ST> {
    state: ST,
}

impl<ST> GetClaimsLayer<ST> {
    pub fn new(state: ST) -> Self {
        Self { state }
    }
}

impl<S, ST> Layer<S> for GetClaimsLayer<ST>
where
    ST: Clone,
{
    type Service = GetClaims<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        GetClaims::new(inner, self.state.clone())
    }
}

#[derive(Debug, Clone)]
pub struct GetClaims<S, ST> {
    inner: S,
    state: ST,
}

impl<S, ST> GetClaims<S, ST> {
    fn new(inner: S, state: ST) -> Self {
        GetClaims { inner, state }
    }
}

impl<S, B, ST> Service<Request<B>> for GetClaims<S, ST>
where
    S: Service<Request<B>>,
    ST: StateTrait,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let header = match request.headers().typed_get::<Authorization<Bearer>>() {
            Some(header) => header,
            None => {
                return ResponseFuture {
                    future: self.inner.call(request),
                    span: None,
                }
            }
        };

        let span = match self.state.iam().get_claims(header.token()) {
            Ok(claims) => {
                let span = Some(info_span!("claims", user_id = claims.subject.to_string()));
                request.extensions_mut().insert(claims);
                span
            }
            Err(_) => None,
        };

        ResponseFuture {
            future: self.inner.call(request),
            span,
        }
    }
}

#[pin_project::pin_project]
#[derive(Debug, Clone)]
pub struct ResponseFuture<F> {
    #[pin]
    future: F,
    span: Option<Span>,
}

impl<F> Future for ResponseFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        if let Some(span) = this.span {
            let _enter = span.enter();
            this.future.poll(cx)
        } else {
            this.future.poll(cx)
        }
    }
}
