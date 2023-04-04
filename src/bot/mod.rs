use std::collections::HashSet;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage},
        UpdateHandler,
    },
    prelude::*,
    types::{Update, UserId},
    utils::command::BotCommands,
};
use tracing::warn;

use crate::api::{Api, Img2ImgRequest, Txt2ImgRequest};

mod handlers;
mod helpers;
use handlers::*;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum State {
    #[default]
    New,
    Ready {
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
    SettingsTxt2Img {
        selection: Option<String>,
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
    SettingsImg2Img {
        selection: Option<String>,
        txt2img: Txt2ImgRequest,
        img2img: Img2ImgRequest,
    },
}

impl State {
    fn new_with_defaults(txt2img: Txt2ImgRequest, img2img: Img2ImgRequest) -> Self {
        Self::Ready { txt2img, img2img }
    }
}

fn default_txt2img() -> Txt2ImgRequest {
    Txt2ImgRequest {
        styles: Some(Vec::new()),
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
        restore_faces: Some(false),
        tiling: Some(false),
        negative_prompt: Some("".to_owned()),
        ..Default::default()
    }
}

fn default_img2img() -> Img2ImgRequest {
    Img2ImgRequest {
        denoising_strength: Some(0.75),
        styles: Some(Vec::new()),
        seed: Some(-1),
        sampler_index: Some("Euler".to_owned()),
        batch_size: Some(1),
        n_iter: Some(1),
        steps: Some(50),
        cfg_scale: Some(7.0),
        width: Some(512),
        height: Some(512),
        restore_faces: Some(false),
        tiling: Some(false),
        negative_prompt: Some("".to_owned()),
        resize_mode: Some(1),
        ..Default::default()
    }
}

type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

type DiffusionDialogue = Dialogue<State, ErasedStorage<State>>;

#[derive(Clone, Debug)]
pub(crate) struct ConfigParameters {
    allowed_users: HashSet<UserId>,
    client: reqwest::Client,
    api: Api,
    txt2img_defaults: Txt2ImgRequest,
    img2img_defaults: Img2ImgRequest,
}

pub async fn run_bot(
    api_key: String,
    allowed_users: Vec<u64>,
    db_path: Option<String>,
    sd_api_url: String,
    txt2img_defaults: Option<Txt2ImgRequest>,
    img2img_defaults: Option<Img2ImgRequest>,
) -> anyhow::Result<()> {
    let storage: DialogueStorage = if let Some(path) = db_path {
        SqliteStorage::open(&path, Json)
            .await
            .context("failed to open db")?
            .erase()
    } else {
        InMemStorage::new().erase()
    };

    let bot = Bot::new(api_key);

    let allowed_users = allowed_users.into_iter().map(UserId).collect();

    let client = reqwest::Client::new();

    let api = Api::new_with_client_and_url(client.clone(), sd_api_url)
        .context("Failed to initialize sd api")?;

    let parameters = ConfigParameters {
        allowed_users,
        client,
        api,
        txt2img_defaults: txt2img_defaults.unwrap_or(default_txt2img()),
        img2img_defaults: img2img_defaults.unwrap_or(default_img2img()),
    };

    bot.set_my_commands(UnauthenticatedCommands::bot_commands())
        .scope(teloxide::types::BotCommandScope::Default)
        .await?;

    bot.set_my_commands(SettingsCommands::bot_commands())
        .scope(teloxide::types::BotCommandScope::Default)
        .await?;

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![parameters, storage])
        .default_handler(|upd| async move {
            warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<anyhow::Error> {
    let auth_filter = dptree::filter(|cfg: ConfigParameters, upd: Update| {
        upd.user()
            .map(|user| cfg.allowed_users.contains(&user.id))
            .unwrap_or_default()
    });

    let unauth_command_handler = Update::filter_message().chain(
        teloxide::filter_command::<UnauthenticatedCommands, _>()
            .endpoint(unauthenticated_commands_handler),
    );

    let authenticated = auth_filter.branch(settings_schema()).branch(image_schema());

    dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(unauth_command_handler)
        .branch(authenticated)
}
