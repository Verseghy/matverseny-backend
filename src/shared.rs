use sea_orm::{ConnectionTrait, DbConn, ConnectOptions, Database};
use tracing::log::LevelFilter;
use std::sync::Arc;

pub trait SharedTrait: Send + Sync + Clone + 'static {
    type Db: ConnectionTrait + Clone;

    fn db(&self) -> &Self::Db;
}

pub struct Shared {
    database: DbConn,
}

impl Shared {
    pub async fn new() -> Arc<Self> {
        Arc::new(Self {
            database: Self::connect_database().await,
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

    fn db(&self) -> &Self::Db {
        &self.database
    }
}
