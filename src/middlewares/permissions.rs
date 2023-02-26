use std::{
    convert::Infallible,
    env,
    task::{Context, Poll},
};

use axum::{
    http::{header::AUTHORIZATION, Request},
    response::{IntoResponse, Response},
};
use futures::{future::BoxFuture, Future};
use serde_json::{json, Value};
use tower::{Layer, Service};

use crate::{error, iam::Claims, StateTrait};

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
            return Box::pin(async {
                Ok(error::NOT_ENOUGH_PERMISSIONS.into_response())
            });
        };

        let json = json!({
            "actions": self.permissions,
            "user": claims.subject,
        });
        let secret = self.state.app_secret().to_owned();

        let future = self.inner.call(req);

        Box::pin(async move {
            let client = reqwest::Client::new();

            let res = client
                .post(format!("{}/v1/decision", env::var("IAM_URL").unwrap()))
                .header(AUTHORIZATION, secret)
                .json(&json)
                .send()
                .await;

            let Ok(res) = res else {
                warn!("failed to send http request: {:?}", res);
                return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response())
            };

            let json = match res.json::<Value>().await {
                Err(err) => {
                    error!("IAM did not return valied json: {:?}", err);
                    return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response());
                }
                Ok(json) => json,
            };

            if let Some(err) = json.get("error") {
                error!("IAM return error: {}, code: {:?}", err, json.get("code"));
                return Ok(error::NOT_ENOUGH_PERMISSIONS.into_response());
            }

            future.await
        })
    }
}
