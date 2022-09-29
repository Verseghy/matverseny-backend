use crate::iam::{Iam, IamTrait};
use std::sync::Arc;
use rand::{
    rngs::{adapter::ReseedingRng, OsRng},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha20Core;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn, TransactionTrait};
use tracing::log::LevelFilter;

pub trait SharedTrait: Send + Sync + Clone + 'static {
    type Db: ConnectionTrait + TransactionTrait + Clone;
    type Iam: IamTrait;
    type Rand: Rng;

    fn db(&self) -> &Self::Db;
    fn iam(&self) -> &Self::Iam;
    fn rng(&self) -> Self::Rand;
}

pub struct Shared {
    database: DbConn,
    iam: Iam,
}

impl Shared {
    pub async fn new() -> Arc<Self> {
        Self::with_database(Self::connect_database().await).await
    }

    pub async fn with_database(conn: DbConn) -> Arc<Self> {
        Arc::new(Self {
            database: conn,
            iam: Iam::new(),
        })
    }

    async fn connect_database() -> DbConn {
        tracing::info!("Trying to connect to database");

        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let mut opts = ConnectOptions::new(url);
        opts.sqlx_logging_level(LevelFilter::Debug);

        let db = Database::connect(opts).await.unwrap();

        tracing::info!("Connected to database");

        db
    }
}

thread_local! {
    static CHACHA_THREAD_RNG: ReseedingRng<ChaCha20Core, OsRng> = {
        let rng = ChaCha20Core::from_entropy();
        ReseedingRng::new(rng, 1024*64, OsRng)
    }
}

impl SharedTrait for Arc<Shared> {
    type Db = DbConn;
    type Iam = Iam;
    type Rand = ReseedingRng<ChaCha20Core, OsRng>;

    fn db(&self) -> &Self::Db {
        &self.database
    }

    fn iam(&self) -> &Self::Iam {
        &self.iam
    }

    fn rng(&self) -> Self::Rand {
        CHACHA_THREAD_RNG.with(|x| x.clone())
    }

}
