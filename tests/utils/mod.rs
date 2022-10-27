pub mod iam;
pub mod macros;
pub mod prelude;
mod request;
mod response;
mod team;
mod user;

use dotenvy::dotenv;
use http::StatusCode;
use matverseny_backend::Shared;
use migration::MigratorTrait;
use request::*;
use reqwest::Client;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn, Statement};
use serde_json::{json, Value};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use team::Team;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tracing::log::LevelFilter;
use user::*;
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
        let mut opts = ConnectOptions::new(DEFAULT_URL.to_owned());
        opts.sqlx_logging_level(LevelFilter::Debug);

        let conn = Database::connect(opts)
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

        let mut opts = ConnectOptions::new(format!("{}/{}", DEFAULT_URL, database));
        opts.sqlx_logging_level(LevelFilter::Debug);

        let conn2 = Database::connect(opts)
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

        User::new(user.id, user.email, user.access_token, self.clone())
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

        Team::new(
            json["id"].as_str().expect("no id").to_owned(),
            owner.clone(),
            self.clone(),
        )
    }

    #[allow(dead_code)]
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.inner
                .client
                .post(format!("http://{}{}", self.inner.addr, url)),
        )
    }

    #[allow(dead_code)]
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.inner
                .client
                .patch(format!("http://{}{}", self.inner.addr, url)),
        )
    }

    #[allow(unused)]
    pub fn socket(&self, url: &str) -> SocketRequestBuilder {
        let uri = format!("ws://{}{}", self.inner.addr, url);

        SocketRequestBuilder::new(
            http::request::Builder::new()
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
        )
    }
}

#[allow(unused)]
#[track_caller]
pub fn get_socket_message(
    message: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
) -> Value {
    if let Some(Ok(Message::Text(message))) = message {
        serde_json::from_str(&message).expect("not json")
    } else {
        panic!("not text");
    }
}
