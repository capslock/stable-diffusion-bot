use anyhow::{anyhow, Context};
use clap::Parser;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
use stable_diffusion_bot::{
    ApiType, ComfyUIConfig, MessageParameters, SettingsParameters, StableDiffusionBotBuilder,
    UiParameters,
};
use tracing::{info, metadata::LevelFilter};
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
    /// Output logs directly to systemd
    #[arg(long, default_value = "false")]
    log_to_systemd: bool,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    api_key: String,
    allowed_users: Vec<i64>,
    db_path: Option<String>,
    sd_api_url: String,
    api_type: Option<ApiType>,
    txt2img: Option<Txt2ImgRequest>,
    img2img: Option<Img2ImgRequest>,
    allow_all_users: Option<bool>,
    comfyui: Option<ComfyUIConfig>,
    administrator_users: Option<Vec<i64>>,
    settings: Option<Settings>,
    ui: Option<Ui>,
    messages: Option<Messages>,
    start_message: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Settings {
    disable_user_settings: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Ui {
    hide_rerun_button: Option<bool>,
    hide_reuse_button: Option<bool>,
    hide_settings_button: Option<bool>,
    hide_all_buttons: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct Messages {
    hide_generation_info: Option<bool>,
    generation_info: Option<Vec<GenerationInfo>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
enum GenerationInfo {
    Prompt,
    AllPrompts,
    NegativePrompt,
    AllNegativePrompts,
    Seed,
    AllSeeds,
    Subseed,
    AllSubseeds,
    SubseedStrength,
    Width,
    Height,
    SamplerName,
    CfgScale,
    Steps,
    BatchSize,
    RestoreFaces,
    FaceRestorationModel,
    SdModelName,
    SdModelHash,
    SdVaeName,
    SdVaeHash,
    SeedResizeFromW,
    SeedResizeFromH,
    DenoisingStrength,
    ExtraGenerationParams,
    IndexOfFirstImage,
    Infotexts,
    Styles,
    JobTimestamp,
    ClipSkip,
    IsUsingInpaintingConditioning,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let registry = tracing_subscriber::registry();
    let layer = {
        #[cfg(target_os = "linux")]
        if args.log_to_systemd && daemon::booted() {
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
        if args.log_to_systemd {
            return Err(anyhow!("Systemd logging is not supported on this platform"));
        } else {
            tracing_subscriber::fmt::layer().pretty().with_target(true)
        }
    };

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .context("Failed to parse filter from env")?;

    registry.with(filter).with(layer).init();

    let config: Config = args
        .config
        .iter()
        .fold(Figment::new(), |f, path| f.admerge(Toml::file(path)))
        .admerge(Env::prefixed("SD_TELEGRAM_"))
        .extract()
        .context("Invalid configuration")?;

    info!(?config);

    let settings = SettingsParameters {
        disable_user_settings: config
            .settings
            .and_then(|s| s.disable_user_settings)
            .unwrap_or_default(),
    };

    let mut ui = UiParameters {
        hide_rerun_button: config
            .ui
            .as_ref()
            .and_then(|s| s.hide_rerun_button)
            .unwrap_or_default(),
        hide_reuse_button: config
            .ui
            .as_ref()
            .and_then(|s| s.hide_reuse_button)
            .unwrap_or_default(),
        hide_settings_button: config
            .ui
            .as_ref()
            .and_then(|s| s.hide_settings_button)
            .unwrap_or_default(),
        hide_all_buttons: config
            .ui
            .and_then(|s| s.hide_all_buttons)
            .unwrap_or_default(),
    };

    ui.hide_all_buttons |= ui.hide_rerun_button && ui.hide_reuse_button && ui.hide_settings_button;

    let messages = MessageParameters {
        hide_generation_info: config
            .messages
            .as_ref()
            .and_then(|s| s.hide_generation_info)
            .unwrap_or_default(),
    };

    StableDiffusionBotBuilder::new(
        config.api_key,
        config.allowed_users,
        config.sd_api_url,
        config.api_type.unwrap_or_default(),
        config.allow_all_users.unwrap_or_default(),
    )
    .db_path(config.db_path)
    .txt2img_defaults(config.txt2img.unwrap_or_default())
    .img2img_defaults(config.img2img.unwrap_or_default())
    .comfyui_config(config.comfyui.unwrap_or_default())
    .administrator_users(config.administrator_users.unwrap_or_default())
    .configure_settings(settings)
    .configure_ui(ui)
    .configure_messages(messages)
    .build()
    .await?
    .run()
    .await?;

    Ok(())
}
