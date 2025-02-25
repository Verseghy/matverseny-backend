use crate::{UserLike, response::TestResponse};
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub struct SocketRequestBuilder {
    builder: http::request::Builder,
}

impl SocketRequestBuilder {
    pub(crate) fn new(builder: http::request::Builder) -> Self {
        SocketRequestBuilder { builder }
    }

    pub async fn start(self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let request = self.builder.body(()).expect("failed to create request");
        let (stream, _reponse) = tokio_tungstenite::connect_async(request)
            .await
            .expect("failed to create websocket");
        stream
    }

    pub fn into_inner(self) -> http::request::Builder {
        self.builder
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

impl RequestBuilder {
    pub(crate) fn new(builder: reqwest::RequestBuilder) -> Self {
        RequestBuilder { builder }
    }

    pub async fn send(self) -> TestResponse {
        TestResponse::new(self.builder.send().await.expect("failed to send request"))
    }

    pub fn json<T>(mut self, value: &T) -> RequestBuilder
    where
        T: Serialize,
    {
        self.builder = self.builder.json(value);
        self
    }

    pub fn user(mut self, user: &impl UserLike) -> RequestBuilder {
        self.builder = self.builder.bearer_auth(user.access_token());
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> RequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);
        self
    }
}
