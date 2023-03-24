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
};
use tracing::warn;

use crate::api::{Api, Img2ImgRequest, Txt2ImgRequest};

mod handlers;
mod helpers;
use handlers::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum State {
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

impl Default for State {
    fn default() -> Self {
        Self::Ready {
            txt2img: Txt2ImgRequest {
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
            },
            img2img: Img2ImgRequest {
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
                ..Default::default()
            },
        }
    }
}

type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

type DiffusionDialogue = Dialogue<State, ErasedStorage<State>>;

#[derive(Clone, Debug)]
pub(crate) struct ConfigParameters {
    allowed_users: HashSet<UserId>,
    client: reqwest::Client,
    api: Api,
}

pub async fn run_bot(
    api_key: String,
    allowed_users: Vec<u64>,
    db_path: Option<String>,
    sd_api_url: String,
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
    };

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
