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
use sea_orm::{ConnectOptions, Database, DbConn};
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use testcontainers::{runners::AsyncRunner, ContainerAsync, ImageExt};
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

async fn setup_iam() -> (App, Iam, libiam::testing::Database) {
    let db = testing::Database::connect("mysql://iam:secret@127.0.0.1:3306/iam").await;
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

async fn setup_database() -> (ContainerAsync<Postgres>, DbConn) {
    let container = Postgres::default()
        .with_name(format!("{REGISTRY}/library/postgres"))
        .with_tag("16")
        .start()
        .await
        .unwrap();

    let connection_string = format!(
        "postgres://postgres:postgres@{}:{}/postgres",
        container.get_host().await.unwrap(),
        container.get_host_port_ipv4(5432).await.unwrap(),
    );

    let opts = ConnectOptions::new(connection_string);
    let db = Database::connect(opts).await.unwrap();

    migration::Migrator::fresh(&db)
        .await
        .expect("failed to apply migrations");

    (container, db)
}

async fn setup_nats() -> ContainerAsync<Nats> {
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

    // TODO: fix this
    env::set_var("NATS_URL", connection_string);

    container
}

async fn setup_backend(app: App, db: DbConn) -> SocketAddr {
    let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
    let state = State::with_database(app, db).await;

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

    let (iam, db, nats) = tokio::join!(setup_iam(), setup_database(), setup_nats());

    let (app, iam, iam_db) = iam;
    let (container, db) = db;
    let nats = nats;

    let addr = setup_backend(app, db).await;

    Env {
        addr,
        client: Client::new(),
        iam,
        iam_db,
        team_num: Arc::new(AtomicU64::new(0)),
        _container: Arc::new(container),
        _nats: Arc::new(nats),
    }
}

#[derive(Clone)]
pub struct Env {
    pub addr: SocketAddr,
    pub client: Client,
    pub iam: Iam,
    pub iam_db: libiam::testing::Database,
    pub team_num: Arc<AtomicU64>,
    _container: Arc<ContainerAsync<Postgres>>,
    _nats: Arc<ContainerAsync<Nats>>,
}

impl Drop for Env {
    fn drop(&mut self) {
        // TODO: This is a HACK, remove this when possible:
        //       `libiam::testing::Database`'s Drop is dropping a tokio runtime in the current thread,
        //       which is not possible when inside another async runtime. All tests starts it's own
        //       runtime so all tests would panic.
        //       This causes a leak, but after this struct is dropped, the whole process exists so
        //       this shouldn't be a problem.
        #[allow(clippy::mem_forget)]
        std::mem::forget(self.iam_db.clone());
    }
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