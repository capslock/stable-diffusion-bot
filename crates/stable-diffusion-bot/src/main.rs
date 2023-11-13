use anyhow::Context;
use clap::Parser;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
use stable_diffusion_bot::StableDiffusionBotBuilder;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{prelude::*, EnvFilter};

use std::path::PathBuf;

#[cfg(target_os = "linux")]
use libsystemd::daemon;

#[derive(Parser, Debug)]
struct Args {
    /// Path to the configuration file
    #[arg(
        short,
        long,
        value_parser = clap::value_parser!(PathBuf),
        default_value = "config.toml"
    )]
    config: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    api_key: String,
    allowed_users: Vec<i64>,
    db_path: Option<String>,
    sd_api_url: String,
    txt2img: Option<Txt2ImgRequest>,
    img2img: Option<Img2ImgRequest>,
    allow_all_users: Option<bool>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = tracing_subscriber::registry();
    let layer = {
        #[cfg(target_os = "linux")]
        if daemon::booted() {
            tracing_journald::layer()
                .context("tracing_journald layer")?
                .boxed()
        } else {
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(true)
                .boxed()
        }
        #[cfg(not(target_os = "linux"))]
        tracing_subscriber::fmt::layer().pretty().with_target(true)
    };

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .context("Failed to parse filter from env")?;

    registry.with(filter).with(layer).init();

    let args = Args::parse();

    let config: Config = args
        .config
        .iter()
        .fold(Figment::new(), |f, path| f.admerge(Toml::file(path)))
        .admerge(Env::prefixed("SD_TELEGRAM_"))
        .extract()
        .context("Invalid configuration")?;

    StableDiffusionBotBuilder::new(
        config.api_key,
        config.allowed_users,
        config.sd_api_url,
        config.allow_all_users.unwrap_or_default(),
    )
    .db_path(config.db_path)
    .txt2img_defaults(config.txt2img.unwrap_or_default())
    .img2img_defaults(config.img2img.unwrap_or_default())
    .build()
    .await?
    .run()
    .await?;

    Ok(())
}
