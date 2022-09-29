pub mod iam;

#[allow(unused_imports)]
pub(crate) mod prelude {
    pub(crate) use super::{assert_error, App};
    pub use futures::{Stream, StreamExt};
    pub use http::StatusCode;
    pub use matverseny_backend::error;
    pub use serde_json::{json, Value};
}

use dotenvy::dotenv;
use futures::StreamExt;
use http::StatusCode;
use matverseny_backend::Shared;
use migration::MigratorTrait;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client,
};
use sea_orm::{ConnectionTrait, Database, DbConn, Statement};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};
use uuid::Uuid;

const DEFAULT_URL: &str = "postgres://matverseny:secret@127.0.0.1:5432";

pub struct AppInner {
    _database: String,
    _db_conn: DbConn,
    _join_handle: JoinHandle<()>,
    client: Client,
    addr: SocketAddr,
}

#[derive(Clone)]
pub struct App {
    inner: Arc<AppInner>,
}

pub struct Team {
    id: String,
    owner: User,
    app: App,
}

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub access_token: String,
    app: App,
}

impl User {
    pub async fn join(&self, code: &str) {
        let res = self
            .app
            .post("/team/join")
            .user(self)
            .json(&json!({
                "code": code,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::OK);
    }
}

pub trait UserLike {
    fn access_token(&self) -> &String;
}

impl UserLike for User {
    fn access_token(&self) -> &String {
        &self.access_token
    }
}

impl Team {
    #[allow(unused)]
    pub async fn get_code(&self) -> String {
        let mut socket = self.app.socket("/ws").user(&self.owner).start().await;
        let message = socket.next().await;

        if let Some(Ok(Message::Text(message))) = message {
            let value: Value = serde_json::from_str(&message).expect("not json");

            assert!(value.is_object());
            assert!(value["event"].is_string());
            assert_eq!(value["event"].as_str().unwrap(), "TEAM_INFO");

            value["data"]["code"].as_str().expect("no code").to_owned()
        } else {
            panic!("not text");
        }
    }
}

impl App {
    pub async fn new() -> Self {
        dotenv().ok();

        let (conn, conn2, database) = Self::setup_database().await;

        let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
        let listener = TcpListener::bind(addr).expect("failed to bind tcp listener");
        let addr = listener.local_addr().unwrap();
        let shared = Shared::with_database(conn2).await;

        let join_handle = tokio::spawn(matverseny_backend::run(listener, shared));

        let inner = AppInner {
            _database: database,
            _db_conn: conn,
            _join_handle: join_handle,
            client: Client::new(),
            addr,
        };

        App {
            inner: Arc::new(inner),
        }
    }

    async fn setup_database() -> (DbConn, DbConn, String) {
        let conn = Database::connect(DEFAULT_URL)
            .await
            .expect("failed to connect to database");

        let database = Uuid::new_v4()
            .hyphenated()
            .encode_lower(&mut Uuid::encode_buffer())
            .to_owned();

        conn.execute(Statement::from_string(
            conn.get_database_backend(),
            format!("create database \"{}\"", database),
        ))
        .await
        .expect("failed to create database");

        let conn2 = Database::connect(format!("{}/{}", DEFAULT_URL, database))
            .await
            .expect("failed to connect to database");

        migration::Migrator::up(&conn2, None)
            .await
            .expect("failed to apply migrations");

        (conn, conn2, database)
    }

    #[allow(unused)]
    pub async fn register_user(&self) -> User {
        let user = iam::register_user().await;

        let res = self
            .post("/register")
            .user(&user)
            .json(&json!({
                "school": "Test School",
                "class": 9,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        User {
            id: user.id,
            email: user.email,
            access_token: user.access_token,
            app: self.clone(),
        }
    }

    #[allow(unused)]
    pub async fn create_team(&self, owner: &User) -> Team {
        static TEAM_COUNT: AtomicU64 = AtomicU64::new(0);

        let res = self
            .post("/team/create")
            .user(owner)
            .json(&json!({
                "name": format!("Team-{}", TEAM_COUNT.fetch_add(1, Ordering::Relaxed))
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let json: Value = res.json().await;

        Team {
            id: json["id"].as_str().expect("no id").to_owned(),
            owner: owner.clone(),
            app: self.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self
                .inner
                .client
                .post(format!("http://{}{}", self.inner.addr, url)),
        }
    }

    #[allow(unused)]
    pub fn socket(&self, url: &str) -> SocketRequestBuilder {
        let uri = format!("ws://{}{}", self.inner.addr, url);

        SocketRequestBuilder {
            builder: http::request::Builder::new()
                .method("GET")
                .header(http::header::HOST, self.inner.addr.to_string())
                .header(http::header::CONNECTION, "Upgrade")
                .header(http::header::UPGRADE, "websocket")
                .header(http::header::SEC_WEBSOCKET_VERSION, "13")
                .header(
                    http::header::SEC_WEBSOCKET_KEY,
                    tokio_tungstenite::tungstenite::handshake::client::generate_key(),
                )
                .uri(uri),
        }
    }
}

#[derive(Debug)]
pub struct SocketRequestBuilder {
    builder: http::request::Builder,
}

#[allow(unused)]
impl SocketRequestBuilder {
    pub fn user(mut self, user: &impl UserLike) -> SocketRequestBuilder {
        self.builder = self.builder.header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user.access_token()),
        );
        self
    }

    pub async fn start(mut self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let request = self.builder.body(()).expect("failed to create request");
        let (stream, _reponse) = tokio_tungstenite::connect_async(request)
            .await
            .expect("failed to create websocket");
        stream
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

#[allow(unused)]
impl RequestBuilder {
    pub async fn send(self) -> TestResponse {
        TestResponse {
            response: self.builder.send().await.expect("failed to send request"),
        }
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

    #[allow(dead_code)]
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

#[derive(Debug)]
pub struct TestResponse {
    response: reqwest::Response,
}

#[allow(unused)]
impl TestResponse {
    pub async fn json<T: DeserializeOwned>(self) -> T {
        self.response
            .json()
            .await
            .expect("failed to deserialize to json")
    }

    pub fn status(&self) -> StatusCode {
        self.response.status()
    }
}

#[allow(unused_macros)]
macro_rules! assert_error {
    ($res:expr, $error:expr) => {{
        assert_eq!($res.status(), $error.status());

        let res_json: serde_json::Value = $res.json().await;
        assert_eq!(res_json["code"], $error.code());
    }};
}

pub(crate) use assert_error;
