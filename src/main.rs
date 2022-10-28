use matverseny_backend::State;
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
    let state = State::new().await;

    matverseny_backend::run(listener, state).await;
}
