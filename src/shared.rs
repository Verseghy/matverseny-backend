use crate::iam::{Iam, IamTrait};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn};
use std::sync::Arc;
use tracing::log::LevelFilter;

pub trait SharedTrait: Send + Sync + Clone + 'static {
    type Db: ConnectionTrait + Clone;
    type Iam: IamTrait;
    type Rand: Rng + Clone;

    fn db(&self) -> &Self::Db;
    fn iam(&self) -> &Self::Iam;
    fn rng(&self) -> &Self::Rand;
}

pub struct Shared {
    database: DbConn,
    iam: Iam,
    rand: StdRng,
}

impl Shared {
    pub async fn new() -> Arc<Self> {
        Self::with_database(Self::connect_database().await).await
    }

    pub async fn with_database(conn: DbConn) -> Arc<Self> {
        Arc::new(Self {
            database: conn,
            iam: Iam::new(),
            rand: StdRng::from_entropy(),
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

impl SharedTrait for Arc<Shared> {
    type Db = DbConn;
    type Iam = Iam;
    type Rand = StdRng;

    fn db(&self) -> &Self::Db {
        &self.database
    }

    fn iam(&self) -> &Self::Iam {
        &self.iam
    }

    fn rng(&self) -> &Self::Rand {
        &self.rand
    }
}
