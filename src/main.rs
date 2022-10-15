use matverseny_backend::Shared;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .with_line_number(true)
        .init();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3002));

    let listener = TcpListener::bind(addr).expect("failed to bind tcp listener");
    let shared = Shared::new().await;

    matverseny_backend::run(listener, shared).await;
}
