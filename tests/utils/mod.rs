pub mod iam;

use dotenvy::dotenv;
use http::StatusCode;
use iam::User;
use matverseny_backend::Shared;
use migration::MigratorTrait;
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client,
};
use sea_orm::{ConnectionTrait, Database, DbConn, Statement};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use tokio::task::JoinHandle;
use uuid::Uuid;

const DEFAULT_URL: &str = "postgres://matverseny:secret@127.0.0.1:5432";

pub struct App {
    _database: String,
    _db_conn: DbConn,
    _join_handle: JoinHandle<()>,
    client: Client,
    addr: SocketAddr,
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

        App {
            _database: database,
            _db_conn: conn,
            _join_handle: join_handle,
            client: Client::new(),
            addr,
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

        user
    }

    #[allow(dead_code)]
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder {
            builder: self.client.post(format!("http://{}{}", self.addr, url)),
        }
    }
}

#[derive(Debug)]
pub struct RequestBuilder {
    builder: reqwest::RequestBuilder,
}

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

    pub fn user(self, user: &iam::User) -> RequestBuilder {
        self.header(
            http::header::AUTHORIZATION,
            format!("Bearer {}", user.access_token),
        )
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
