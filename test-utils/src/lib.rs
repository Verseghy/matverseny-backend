mod db;
pub mod iam;
pub mod macros;
pub mod prelude;
mod request;
mod response;
mod team;
mod user;

use db::Database;
use dotenvy::dotenv;
use http::StatusCode;
use libiam::testing::actions::{assign_action_to_app, ensure_action};
pub use macros::macro_support;
use matverseny_backend::State;
use request::*;
use reqwest::Client;
use serde_json::{Value, json};
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use team::Team;
use tokio::{
    net::TcpListener,
    sync::{OnceCell, oneshot},
};
use tokio_tungstenite::tungstenite::Message;
use user::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppInner {
    addr: SocketAddr,
    db: Database,
}

#[derive(Clone, Debug)]
pub struct App {
    inner: Arc<AppInner>,
}

impl App {
    pub async fn new() -> Self {
        dotenv().ok();

        let (tx, rx) = oneshot::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(async {
                tracing::trace!("starting app thread");

                let iam = iam::get_iam();
                let iam_db = iam::get_db().await;
                let (_, secret) = libiam::testing::apps::create_app(iam_db, &uuid()).await;
                let iam_app = libiam::App::login(iam, &secret).await.unwrap();

                tracing::trace!("creating actions");

                ensure_action(iam_db, "mathcompetition.problems", true).await;
                ensure_action(iam_db, "mathcompetition.admin", true).await;

                tracing::trace!("assigning actions to app");

                assign_action_to_app(iam_db, "iam.policy.assign", &iam_app.id()).await;
                assign_action_to_app(iam_db, "iam.user.get", &iam_app.id()).await;

                tracing::trace!("setting up database");

                let conn = Database::setup().await;

                tracing::trace!("binding socket");

                let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
                let listener = TcpListener::bind(addr)
                    .await
                    .expect("failed to bind tcp listener");
                let state = State::with_database(iam_app, conn.conn()).await;

                let inner = Arc::new(AppInner {
                    addr: listener.local_addr().unwrap(),
                    db: conn,
                });

                tx.send(inner).unwrap();

                tracing::trace!("starting app");

                matverseny_backend::run(listener, state).await.unwrap();
            });
        });

        let app = App {
            inner: rx.await.unwrap(),
        };

        app.clean_database().await;

        app
    }

    pub async fn clean_database(&self) {
        self.inner.db.clean().await;
    }

    pub async fn register_user(&self) -> User {
        let user = iam::register_user().await;

        let res = self
            .post("/v1/register")
            .user(&user)
            .json(&json!({
                "school": "Test School",
                "class": 9,
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        let token = user.access_token().to_owned();
        User::new(user.id(), user.email, token, self.clone())
    }

    pub async fn create_team(&self, owner: &User) -> Team {
        static TEAM_COUNT: AtomicU64 = AtomicU64::new(0);

        let number = TEAM_COUNT.fetch_add(1, Ordering::Relaxed);

        let res = self
            .post("/v1/team/create")
            .user(owner)
            .json(&json!({
                "name": format!("Team-{}", number)
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        Team::new(owner.clone(), self.clone(), number)
    }

    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().get(format!("http://{}{}", self.inner.addr, url)))
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().post(format!("http://{}{}", self.inner.addr, url)))
    }

    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().patch(format!("http://{}{}", self.inner.addr, url)))
    }

    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().delete(format!("http://{}{}", self.inner.addr, url)))
    }

    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(Client::new().put(format!("http://{}{}", self.inner.addr, url)))
    }

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

#[track_caller]
pub fn get_socket_message(
    message: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
) -> Value {
    tracing::debug!("socket  message: {message:?}");
    if let Some(Ok(Message::Text(message))) = message {
        serde_json::from_str(&message).expect("not json")
    } else {
        panic!("not text");
    }
}

pub async fn get_cached_app() -> &'static App {
    static APP: OnceCell<App> = OnceCell::const_new();
    APP.get_or_init(App::new).await
}

pub fn uuid() -> String {
    Uuid::new_v4()
        .as_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_owned()
}
