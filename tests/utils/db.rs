use migration::MigratorTrait;
use sea_orm::{ConnectOptions, DbConn};
use tokio::sync::{mpsc, oneshot};
use tracing::log::LevelFilter;

const DEFAULT_URL: &str = "postgres://matverseny:secret@127.0.0.1:5432/matverseny";

#[derive(Debug)]
pub struct Database {
    conn: DbConn,
    channel: mpsc::Sender<oneshot::Sender<()>>,
}

impl Database {
    pub async fn setup() -> Self {
        let (tx, mut rx) = mpsc::channel::<oneshot::Sender<()>>(128);
        let (conn_tx, conn_rx) = oneshot::channel();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            rt.block_on(async move {
                let mut opts = ConnectOptions::new(DEFAULT_URL.to_owned());
                opts.sqlx_logging_level(LevelFilter::Debug);

                let conn = sea_orm::Database::connect(opts)
                    .await
                    .expect("failed to connect to database");

                conn_tx.send(conn.clone()).unwrap();

                while let Some(tx) = rx.recv().await {
                    migration::Migrator::fresh(&conn)
                        .await
                        .expect("failed to apply migrations");

                    tx.send(()).unwrap();
                }
            })
        });

        Self {
            conn: conn_rx.await.unwrap(),
            channel: tx,
        }
    }

    pub async fn clean(&self) {
        let (tx, rx) = oneshot::channel();
        self.channel.send(tx).await.unwrap();
        rx.await.unwrap();
    }

    pub fn conn(&self) -> DbConn {
        self.conn.clone()
    }
}
