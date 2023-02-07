use matverseny_backend::State;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_line_number(true).with_filter(env_filter))
        .init();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3002));

    let listener = TcpListener::bind(addr).expect("failed to bind tcp listener");
    let state = State::new().await;

    matverseny_backend::run(listener, state).await;
}
