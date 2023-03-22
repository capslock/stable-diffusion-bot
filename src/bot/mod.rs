use std::collections::HashSet;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage},
        UpdateHandler,
    },
    dptree::case,
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
    Next,
}

impl Default for State {
    fn default() -> Self {
        Self::Ready {
            txt2img: Default::default(),
            img2img: Default::default(),
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
    let auth_filter = dptree::filter(|cfg: ConfigParameters, msg: Message| {
        msg.from()
            .map(|user| cfg.allowed_users.contains(&user.id))
            .unwrap_or_default()
    });

    let unauth_command_handler = teloxide::filter_command::<UnauthenticatedCommands, _>()
        .endpoint(unauthenticated_commands_handler);

    let auth_command_handler = auth_filter
        .clone()
        .filter_command::<AuthenticatedCommands>()
        .endpoint(
            |msg: Message, bot: Bot, cmd: AuthenticatedCommands| async move {
                match cmd {
                    AuthenticatedCommands::Set { value } => {
                        bot.send_message(msg.chat.id, format!("{value}")).await?;
                        Ok(())
                    }
                }
            },
        );

    let message_handler = auth_filter
        .branch(
            Message::filter_photo()
                .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_image)),
        )
        .branch(
            Message::filter_text()
                .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_prompt)),
        );

    let callback_handler = Update::filter_callback_query().branch(
        dptree::filter(|q: CallbackQuery| {
            if let Some(data) = q.data {
                data.starts_with("rerun")
            } else {
                false
            }
        })
        .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_rerun)),
    );

    let handler = Update::filter_message()
        .branch(unauth_command_handler)
        .branch(auth_command_handler)
        .branch(message_handler);

    dialogue::enter::<Update, ErasedStorage<State>, State, _>()
        .branch(handler)
        .branch(callback_handler)
}
