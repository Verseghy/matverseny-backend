use crate::{
    iam::{Iam, IamTrait},
    utils::Problems,
};
use libiam::App;
use rand::{
    rngs::{adapter::ReseedingRng, OsRng},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Core;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn, TransactionTrait};
use std::{env, sync::Arc};
use tracing::log::LevelFilter;

pub trait StateTrait: Send + Sync + Clone + 'static {
    type Db: ConnectionTrait + TransactionTrait + Clone;
    type Iam: IamTrait;
    type Rand: Rng;

    fn db(&self) -> &Self::Db;
    fn iam(&self) -> &Self::Iam;
    fn iam_app(&self) -> &App;
    fn rng(&self) -> Self::Rand;
    fn app_secret(&self) -> &str;
    fn problems(&self) -> Arc<Problems>;
    fn nats(&self) -> async_nats::Client;
}

pub struct State {
    database: DbConn,
    iam: Iam,
    iam_app: App,
    app_secret: String,
    problems: Arc<Problems>,
    nats: async_nats::Client,
}

impl State {
    pub async fn new(iam_app: App) -> Arc<Self> {
        Self::with_database(iam_app, Self::connect_database().await).await
    }

    pub async fn with_database(iam_app: App, conn: DbConn) -> Arc<Self> {
        let nats = Self::connect_nats().await;
        let problems = Problems::new(&conn, nats.clone()).await;
        Arc::new(Self {
            database: conn,
            iam: Iam::new(),
            iam_app,
            app_secret: env::var("IAM_APP_SECRET").expect("IAM_APP_SECRET is not set"),
            problems: Arc::new(problems),
            nats,
        })
    }

    async fn connect_database() -> DbConn {
        info!("Trying to connect to database");

        let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let mut opts = ConnectOptions::new(url);
        opts.sqlx_logging_level(LevelFilter::Debug);

        let db = Database::connect(opts).await.unwrap();

        info!("Connected to database");

        db
    }

    async fn connect_nats() -> async_nats::Client {
        info!("Trying to connect to NATS");

        let url = env::var("NATS_URL").expect("NATS_URL is not set");
        let client = async_nats::connect(url).await.unwrap();

        info!("Connected to NATS");

        client
    }
}

thread_local! {
    static CHACHA_THREAD_RNG: ReseedingRng<ChaCha20Core, OsRng> = {
        let rng = ChaCha20Core::from_entropy();
        ReseedingRng::new(rng, 1024*64, OsRng)
    }
}

impl StateTrait for Arc<State> {
    type Db = DbConn;
    type Iam = Iam;
    type Rand = ReseedingRng<ChaCha20Core, OsRng>;

    fn db(&self) -> &Self::Db {
        &self.database
    }

    fn iam(&self) -> &Self::Iam {
        &self.iam
    }

    fn iam_app(&self) -> &App {
        &self.iam_app
    }

    fn rng(&self) -> Self::Rand {
        CHACHA_THREAD_RNG.with(|x| x.clone())
    }

    fn app_secret(&self) -> &str {
        &self.app_secret
    }

    fn problems(&self) -> Arc<Problems> {
        Arc::clone(&self.problems)
    }

    fn nats(&self) -> async_nats::Client {
        self.nats.clone()
    }
}
