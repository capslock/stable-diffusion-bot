use anyhow::Context;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use tracing::metadata::LevelFilter;
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, FmtSubscriber};

pub mod api;
pub mod bot;

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    api_key: String,
    allowed_users: Vec<u64>,
    db_path: Option<String>,
    sd_api_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder().pretty().with_target(true).finish();

    tracing::subscriber::set_global_default(
        subscriber.with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .from_env_lossy(),
        ),
    )
    .context("setting default subscriber failed")?;

    LogTracer::init()?;

    let config: Config = Figment::new()
        .merge(("config.allowed_users", Vec::<u64>::new()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("SD_TELEGRAM_"))
        .extract()
        .context("Invalid configuration")?;

    bot::run_bot(
        config.api_key,
        config.allowed_users,
        config.db_path,
        config.sd_api_url,
    )
    .await?;

    Ok(())
}
