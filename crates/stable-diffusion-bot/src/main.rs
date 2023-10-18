use anyhow::Context;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
use stable_diffusion_bot::StableDiffusionBotBuilder;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

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
    tracing_log::env_logger::init();

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .context("Failed to parse filter from env")?;

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .pretty()
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;

    let config: Config = Figment::new()
        .merge(Toml::file("/etc/sdbot/config.toml"))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("SD_TELEGRAM_"))
        .extract()
        .context("Invalid configuration")?;

    StableDiffusionBotBuilder::new(
        config.api_key,
        config.allowed_users,
        config.sd_api_url,
        config.allow_all_users.unwrap_or_default(),
    )
    .db_path(config.db_path)
    .txt2img_defaults(config.txt2img)
    .img2img_defaults(config.img2img)
    .build()
    .await?
    .run()
    .await?;

    Ok(())
}
