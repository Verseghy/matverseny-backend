use crate::{
    iam::{Iam, IamTrait},
    utils::topics,
};
use libiam::App;
use rand::{
    rngs::{adapter::ReseedingRng, OsRng},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Core;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    client::DefaultClientContext,
    producer::FutureProducer,
    ClientConfig,
};
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
    fn kafka_producer(&self) -> &FutureProducer;
    fn kafka_admin(&self) -> &AdminClient<DefaultClientContext>;
    fn app_secret(&self) -> &str;
}

pub struct State {
    database: DbConn,
    iam: Iam,
    iam_app: App,
    kafka_producer: FutureProducer,
    kafka_admin: AdminClient<DefaultClientContext>,
    app_secret: String,
}

impl State {
    pub async fn new(iam_app: App) -> Arc<Self> {
        Self::with_database(iam_app, Self::connect_database().await).await
    }

    pub async fn with_database(iam_app: App, conn: DbConn) -> Arc<Self> {
        Arc::new(Self {
            database: conn,
            iam: Iam::new(),
            iam_app,
            kafka_producer: Self::create_kafka_producer(),
            kafka_admin: Self::create_kafka_admin().await,
            app_secret: env::var("IAM_APP_SECRET").expect("IAM_APP_SECRET is not set"),
        })
    }

    fn create_kafka_producer() -> FutureProducer {
        info!("Creating kafka producer");

        let bootstrap_servers =
            env::var("KAFKA_BOOTSTRAP_SERVERS").expect("KAFKA_BOOTSTRAP_SERVERS not set");

        ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .create()
            .expect("failed to create kafka producer")
    }

    async fn create_kafka_admin() -> AdminClient<DefaultClientContext> {
        info!("Creating kafka admin client");

        let bootstrap_servers =
            env::var("KAFKA_BOOTSTRAP_SERVERS").expect("KAFKA_BOOTSRAP_SERVERS not set");

        let admin = ClientConfig::new()
            .set("bootstrap.servers", bootstrap_servers)
            .create::<AdminClient<DefaultClientContext>>()
            .expect("failed to create kafka admin client");

        admin
            .create_topics(
                &[NewTopic::new(
                    &topics::times(),
                    1,
                    TopicReplication::Fixed(1),
                )],
                &AdminOptions::new(),
            )
            .await
            .expect("failed to create times topic");

        admin
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

    fn kafka_producer(&self) -> &FutureProducer {
        &self.kafka_producer
    }

    fn kafka_admin(&self) -> &AdminClient<DefaultClientContext> {
        &self.kafka_admin
    }

    fn app_secret(&self) -> &str {
        &self.app_secret
    }
}
