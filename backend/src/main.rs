use libiam::{App, Iam};
use matverseny_backend::State;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
    process::ExitCode,
};
use tokio::net::TcpListener;
use tracing::{error, level_filters::LevelFilter};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub async fn login() -> anyhow::Result<App> {
    let iam_url = env::var("IAM_URL").inspect_err(|_| error!("IAM_URL is not set"))?;
    let app_secret =
        env::var("IAM_APP_SECRET").inspect_err(|_| error!("IAM_APP_SECRET is not set"))?;

    let iam = Iam::new(&iam_url);

    App::login(&iam, &app_secret)
        .await
        .inspect_err(|_| error!("failed to login into the iam"))
}

#[tokio::main]
async fn main() -> ExitCode {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer().with_line_number(true).with_filter(env_filter))
        .init();

    matverseny_backend::panic::set_hook();

    if run().await.is_err() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

async fn run() -> anyhow::Result<()> {
    let iam_app = login().await?;

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 3002));

    let listener = TcpListener::bind(addr).await?;
    let state = State::new(iam_app).await;

    matverseny_backend::run(listener, state).await
}
