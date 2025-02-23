use std::{
    convert::Infallible,
    env,
    task::{Context, Poll},
};

use axum::{
    http::{Request, header::AUTHORIZATION},
    response::{IntoResponse, Response},
};
use futures::{Future, future::BoxFuture};
use libiam::jwt::Claims;
use serde::Deserialize;
use serde_json::json;
use tower::{Layer, Service};

use crate::{StateTrait, error};

type PermissionList = &'static [&'static str];

#[derive(Debug, Clone)]
pub struct PermissionsLayer<ST> {
    state: ST,
    permissions: PermissionList,
}

impl<ST> PermissionsLayer<ST> {
    pub fn new(state: ST, permissions: PermissionList) -> Self {
        Self { state, permissions }
    }
}

impl<S, ST> Layer<S> for PermissionsLayer<ST>
where
    ST: Clone,
{
    type Service = Permissions<S, ST>;

    fn layer(&self, inner: S) -> Self::Service {
        Permissions::new(self.state.clone(), inner, self.permissions)
    }
}

#[derive(Debug, Clone)]
pub struct Permissions<S, ST> {
    state: ST,
    inner: S,
    permissions: PermissionList,
}

impl<S, ST> Permissions<S, ST> {
    fn new(state: ST, inner: S, permissions: PermissionList) -> Self {
        Permissions {
            state,
            inner,
            permissions,
        }
    }
}

impl<S, B, ST> Service<Request<B>> for Permissions<S, ST>
where
    S: Service<Request<B>, Error = Infallible, Response = Response> + Send,
    S::Future: Future + Send + 'static,
    B: Send + 'static,
    ST: StateTrait,
{
    type Response = Response;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let Some(claims) = req.extensions().get::<Claims>() else {
            return Box::pin(async { Ok(error::NOT_ENOUGH_PERMISSIONS.into_response()) });
        };

        let json = json!({
            "actions": self.permissions,
            "user": claims.sub,
        });
        let token = self.state.iam_app().token().to_owned();

        let future = self.inner.call(req);

        Box::pin(async move {
            let client = reqwest::Client::new();

            let res = client
                .post(format!("{}/v1/decision", env::var("IAM_URL").unwrap()))
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .json(&json)
                .send()
                .await;

            let Ok(res) = res else {
                warn!("failed to send http request: {:?}", res);
                return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response());
            };

            #[derive(Debug, Deserialize)]
            #[allow(unused)]
            struct Response {
                code: String,
                error: String,
            }

            if !res.status().is_success() {
                let status = res.status();
                let text = res.text().await.unwrap();

                if let Ok(err) = serde_json::from_str::<Response>(&text) {
                    error!("IAM returned error: {:?}, status: {}", err, status);
                    return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response());
                }

                error!("text: {}", text);
                return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response());
            }

            future.await
        })
    }
}
