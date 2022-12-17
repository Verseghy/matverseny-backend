pub mod iam;
pub mod macros;
pub mod prelude;
mod request;
mod response;
mod team;
mod user;

use dotenvy::dotenv;
use http::StatusCode;
use matverseny_backend::State;
use migration::MigratorTrait;
use request::*;
use reqwest::Client;
use sea_orm::{ConnectOptions, Database, DbConn};
use serde_json::{json, Value};
use std::{
    net::{Ipv4Addr, SocketAddr, TcpListener},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use team::Team;
use tokio::sync::OnceCell;
use tokio_tungstenite::tungstenite::Message;
use tracing::log::LevelFilter;
use user::*;
use uuid::Uuid;

const DEFAULT_URL: &str = "postgres://matverseny:secret@127.0.0.1:5432/matverseny";

pub struct AppInner {
    addr: SocketAddr,
    db: DbConn,
}

#[derive(Clone)]
pub struct App {
    inner: Arc<AppInner>,
}

impl App {
    #[allow(unused)]
    pub async fn new() -> Self {
        Self::new_inner(false).await
    }

    #[allow(unused)]
    pub async fn new_with_rt() -> Self {
        Self::new_inner(true).await
    }

    async fn new_inner(with_rt: bool) -> Self {
        dotenv().ok();

        let conn = Self::setup_database().await;

        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        let listener = TcpListener::bind(addr).expect("failed to bind tcp listener");
        let addr = listener.local_addr().unwrap();
        let state = State::with_database(conn.clone()).await;

        if with_rt {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");

                rt.block_on(async {
                    matverseny_backend::run(listener, state).await;
                });
            });
        } else {
            tokio::spawn(matverseny_backend::run(listener, state));
        }

        let inner = AppInner { addr, db: conn };

        App {
            inner: Arc::new(inner),
        }
    }

    #[allow(unused)]
    pub async fn clean_database(&self) {
        migration::Migrator::fresh(&self.inner.db)
            .await
            .expect("failed to apply migraitons");
    }

    async fn setup_database() -> DbConn {
        let mut opts = ConnectOptions::new(DEFAULT_URL.to_owned());
        opts.sqlx_logging_level(LevelFilter::Debug);

        let conn = Database::connect(opts)
            .await
            .expect("failed to connect to database");

        migration::Migrator::fresh(&conn)
            .await
            .expect("failed to apply migrations");

        conn
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

        Team::new(owner.clone(), self.clone())
    }

    #[allow(dead_code)]
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().get(format!("http://{}{}", self.inner.addr, url)))
    }

    #[allow(dead_code)]
    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().post(format!("http://{}{}", self.inner.addr, url)))
    }

    #[allow(dead_code)]
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().patch(format!("http://{}{}", self.inner.addr, url)))
    }

    #[allow(dead_code)]
    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().delete(format!("http://{}{}", self.inner.addr, url)))
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

#[allow(unused)]
pub async fn get_cached_app() -> &'static App {
    static APP: OnceCell<App> = OnceCell::const_new();
    APP.get_or_init(App::new_with_rt).await
}

#[allow(unused)]
pub fn uuid() -> String {
    Uuid::new_v4()
        .as_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}
