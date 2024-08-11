use super::{
    iam,
    request::{RequestBuilder, SocketRequestBuilder},
    team::Team,
    user::{User, UserLike},
    uuid,
};
use http::StatusCode;
use libiam::{
    testing::{
        self,
        actions::{assign_action_to_app, ensure_action},
    },
    App, Iam,
};
use matverseny_backend::State;
use migration::MigratorTrait;
use reqwest::Client;
use sea_orm::{Database, DbConn};
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use testcontainers::{
    core::logs::consumer::logging_consumer::LoggingConsumer, runners::AsyncRunner, ContainerAsync,
    ImageExt,
};
use testcontainers_modules::{nats::Nats, postgres::Postgres};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

const REGISTRY: &str = "docker.io";

fn setup_logging() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env_lossy();

    let layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(layer).init();
}

async fn setup_iam() -> (App, Iam, DbConn) {
    tracing::debug!("Setup IAM");

    let db = Database::connect("mysql://iam:secret@127.0.0.1:3306/iam")
        .await
        .unwrap();

    let base = env::var("IAM_URL").unwrap();
    let iam = Iam::new(&base);
    let (_, secret) = testing::apps::create_app(&db, &uuid()).await;
    let app = App::login(&iam, &secret).await.unwrap();

    ensure_action(&db, "mathcompetition.problems", true).await;
    ensure_action(&db, "mathcompetition.admin", true).await;
    assign_action_to_app(&db, "iam.policy.assign", &app.id()).await;
    assign_action_to_app(&db, "iam.user.get", &app.id()).await;

    (app, iam, db)
}

async fn setup_database() -> ContainerAsync<Postgres> {
    tracing::debug!("Starting Postgres");

    let container = Postgres::default()
        .with_name(format!("{REGISTRY}/library/postgres"))
        .with_tag("16")
        .with_log_consumer(LoggingConsumer::new())
        .start()
        .await
        .unwrap();

    tracing::debug!("Postgres started");

    let connection_string = &format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(5432).await.unwrap(),
    );

    env::set_var("DATABASE_URL", connection_string);

    // sleep(Duration::from_secs(5)).await;

    tracing::debug!("Connecting to Postgres at {:?}", connection_string);

    let db = Database::connect(connection_string)
        .await
        .inspect_err(|err| tracing::error!("postgres connect err: {err:?}"))
        .unwrap();

    tracing::debug!("Running migrations");

    migration::Migrator::fresh(&db)
        .await
        .expect("failed to apply migrations");

    container
}

async fn setup_nats() -> ContainerAsync<Nats> {
    tracing::debug!("Starting NATS");

    let container = Nats::default()
        .with_name(format!("{REGISTRY}/library/nats"))
        .with_tag("2")
        .start()
        .await
        .unwrap();

    let connection_string = format!(
        "{}:{}",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(4222).await.unwrap(),
    );

    env::set_var("NATS_URL", connection_string);

    tracing::debug!("NATS started");

    container
}

async fn setup_backend(app: App) -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let state = State::new(app).await;

    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        matverseny_backend::run(listener, state).await.unwrap();
    });

    addr
}

#[allow(unused)]
pub async fn setup() -> Env {
    dotenvy::dotenv().ok();

    setup_logging();

    let (app, iam, iam_db) = setup_iam().await;
    let db_container = setup_database().await;
    let nats_container = setup_nats().await;

    let addr = setup_backend(app).await;

    Env {
        addr,
        client: Client::new(),
        iam,
        iam_db,
        team_num: Arc::new(AtomicU64::new(0)),
        _db_container: Arc::new(db_container),
        _nats_container: Arc::new(nats_container),
    }
}

#[derive(Clone)]
pub struct Env {
    pub addr: SocketAddr,
    pub client: Client,
    pub iam: Iam,
    pub iam_db: DbConn,
    pub team_num: Arc<AtomicU64>,
    _db_container: Arc<ContainerAsync<Postgres>>,
    _nats_container: Arc<ContainerAsync<Nats>>,
}

impl Env {
    fn get_url(&self, url: &str) -> String {
        format!("http://{}{}", self.addr, url)
    }

    #[allow(unused)]
    pub fn get(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.client.get(self.get_url(url)))
    }

    pub fn post(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.client.post(self.get_url(url)))
    }

    #[allow(unused)]
    pub fn patch(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.client.patch(self.get_url(url)))
    }

    #[allow(unused)]
    pub fn delete(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.client.delete(self.get_url(url)))
    }

    #[allow(unused)]
    pub fn put(&self, url: &str) -> RequestBuilder {
        RequestBuilder::new(self.client.put(self.get_url(url)))
    }

    pub fn socket(&self, url: &str) -> SocketRequestBuilder {
        let uri = format!("ws://{}{}", self.addr, url);

        SocketRequestBuilder::new(
            http::request::Builder::new()
                .method("GET")
                .header(http::header::HOST, self.addr.to_string())
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

impl Env {
    #[allow(unused)]
    pub async fn register_user(&self) -> User {
        let user = iam::register_user(self).await;

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

        let token = user.access_token().to_owned();
        User::new(user.id(), user.email, token, self.clone())
    }

    #[allow(unused)]
    pub async fn create_team(&self, owner: &User) -> Team {
        let number = self.team_num.fetch_add(1, Ordering::Relaxed);

        let res = self
            .post("/team/create")
            .user(owner)
            .json(&json!({
                "name": format!("Test Team {number}"),
            }))
            .send()
            .await;

        assert_eq!(res.status(), StatusCode::CREATED);

        Team::new(self, owner.clone(), number)
    }
}
