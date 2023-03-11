use libiam::{App, Iam};
use matverseny_backend::State;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr, TcpListener},
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

pub async fn login() -> App {
    let Ok(iam_url) = env::var("IAM_URL") else {
        panic!("IAM_URL is not set");
    };

    let iam = Iam::new(&iam_url);

    let Ok(app_secret) = env::var("IAM_APP_SECRET") else {
        panic!("IAM_APP_SECRET is not set");
    };

    App::login(&iam, &app_secret)
        .await
        .expect("failed to login into the iam")
}

#[tokio::main]
async fn main() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_line_number(true).with_filter(env_filter))
        .init();

    let iam_app = login().await;

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3002));

    let listener = TcpListener::bind(addr).expect("failed to bind tcp listener");
    let state = State::new(iam_app).await;

    matverseny_backend::run(listener, state).await;
}
