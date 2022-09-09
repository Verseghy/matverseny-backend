use crate::{iam::IamTrait, SharedTrait};
use axum::{
    headers::{
        authorization::{Authorization, Bearer},
        HeaderMapExt,
    },
    http::Request,
};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Span;

#[derive(Debug, Clone)]
pub struct GetClaimsLayer<SH> {
    _marker: PhantomData<*const SH>,
}

impl<SH> GetClaimsLayer<SH> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S, SH> Layer<S> for GetClaimsLayer<SH> {
    type Service = GetClaims<S, SH>;

    fn layer(&self, inner: S) -> Self::Service {
        GetClaims::new(inner)
    }
}

#[derive(Debug, Clone)]
pub struct GetClaims<S, SH> {
    inner: S,
    _marker: PhantomData<*const SH>,
}

unsafe impl<S, SH> Send for GetClaims<S, SH> where S: Send {}

impl<S, SH> GetClaims<S, SH> {
    fn new(inner: S) -> Self {
        GetClaims {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<S, B, SH> Service<Request<B>> for GetClaims<S, SH>
where
    S: Service<Request<B>>,
    SH: SharedTrait,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<B>) -> Self::Future {
        let shared = request.extensions().get::<SH>().expect("no Shared");

        let header = match request.headers().typed_get::<Authorization<Bearer>>() {
            Some(header) => header,
            None => {
                return ResponseFuture {
                    future: self.inner.call(request),
                    span: None,
                }
            }
        };

        let span = match shared.iam().get_claims(header.token()) {
            Ok(claims) => {
                let span = Some(tracing::info_span!("claims", user_id = claims.subject));
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
