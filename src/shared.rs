use crate::iam::{Iam, IamTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DbConn};
use std::sync::Arc;
use tracing::log::LevelFilter;

pub trait SharedTrait: Send + Sync + Clone + 'static {
    type Db: ConnectionTrait + Clone;
    type Iam: IamTrait;

    fn db(&self) -> &Self::Db;
    fn iam(&self) -> &Self::Iam;
}

pub struct Shared {
    database: DbConn,
    iam: Iam,
}

impl Shared {
    pub async fn new() -> Arc<Self> {
        Arc::new(Self {
            database: Self::connect_database().await,
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

impl SharedTrait for Arc<Shared> {
    type Db = DbConn;
    type Iam = Iam;

    fn db(&self) -> &Self::Db {
        &self.database
    }

    fn iam(&self) -> &Self::Iam {
        &self.iam
    }
}