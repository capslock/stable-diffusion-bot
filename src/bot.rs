use std::collections::HashSet;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{
        self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage,
    },
    dptree::case,
    payloads::setters::*,
    prelude::*,
    types::{ChatAction, InputFile, InputMedia, InputMediaPhoto, PhotoSize, Update, UserId},
    utils::command::BotCommands,
};
use tracing::{error, warn};

use crate::api::{Api, Img2ImgRequest, Img2ImgResponse, Txt2ImgRequest, Txt2ImgResponse};

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
struct ConfigParameters {
    allowed_users: HashSet<UserId>,
    client: reqwest::Client,
    api: Api,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Simple commands")]
enum UnauthenticatedCommands {
    #[command(description = "shows this message.")]
    Help,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Authenticated commands")]
enum AuthenticatedCommands {
    #[command(description = "Set the value")]
    Set { value: u64 },
}

async fn unauthenticated_commands_handler(
    cfg: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: UnauthenticatedCommands,
) -> anyhow::Result<()> {
    let text = match cmd {
        UnauthenticatedCommands::Help => {
            if cfg.allowed_users.contains(&msg.from().unwrap().id) {
                format!(
                    "{}\n\n{}",
                    UnauthenticatedCommands::descriptions(),
                    AuthenticatedCommands::descriptions()
                )
            } else if msg.chat.is_group() || msg.chat.is_supergroup() {
                UnauthenticatedCommands::descriptions()
                    .username_from_me(&me)
                    .to_string()
            } else {
                UnauthenticatedCommands::descriptions().to_string()
            }
        }
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
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

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<UnauthenticatedCommands>()
                .endpoint(unauthenticated_commands_handler),
        )
        .branch(
            dptree::filter(|cfg: ConfigParameters, msg: Message| {
                msg.from()
                    .map(|user| cfg.allowed_users.contains(&user.id))
                    .unwrap_or_default()
            })
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
            ),
        )
        .branch(
            Message::filter_photo()
                .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_image))
                .branch(case![State::Next].endpoint(handle_image)),
        )
        .branch(
            Message::filter_text()
                .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_prompt))
                .branch(case![State::Next].endpoint(handle_prompt)),
        );

    let dialogue = dialogue::enter::<Update, ErasedStorage<State>, State, _>().branch(handler);

    Dispatcher::builder(bot, dialogue)
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

async fn handle_image(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (txt2img, mut img2img): (Txt2ImgRequest, Img2ImgRequest),
    msg: Message,
    photo: Vec<PhotoSize>,
) -> anyhow::Result<()> {
    let prompt = if let Some(caption) = msg.caption() {
        caption
    } else {
        return Ok(());
    };

    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    img2img.prompt = Some(prompt.to_owned());
    let photo = if let Some(photo) = photo
        .iter()
        .reduce(|a, p| if a.height > p.height { a } else { p })
    {
        photo
    } else {
        return Ok(());
    };
    let file = bot.get_file(&photo.file.id).send().await?;

    let photo = cfg
        .client
        .get(format!(
            "https://api.telegram.org/file/bot{}/{}",
            bot.token(),
            file.path
        ))
        .send()
        .await?;

    use base64::{engine::general_purpose, Engine as _};

    let photo = general_purpose::STANDARD.encode(photo.bytes().await?);

    img2img.init_images = Some(vec![photo]);

    let resp = cfg.api.img2img()?.send(&img2img).await?;

    let mut images: Vec<_> = resp
        .images
        .iter()
        .map(|i| {
            InputMedia::Photo(InputMediaPhoto::new(InputFile::memory(
                general_purpose::STANDARD
                    .decode(i)
                    .expect("failed to decode!"),
            )))
        })
        .collect();

    match images.len() {
        1 => {
            if let Some(image) = images.pop() {
                bot.send_photo(msg.chat.id, image.into())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .caption(
                        message_from_resp(prompt, &resp)
                            .context("Failed to build message from response")?,
                    )
                    .await?;
            }
        }
        2.. => {
            if let Some(InputMedia::Photo(image)) = images.get_mut(0) {
                image.caption = Some(
                    message_from_resp(prompt, &resp)
                        .context("Failed to build message from response")?,
                );
            }
            bot.send_media_group(msg.chat.id, images).await?;
        }
        _ => {
            error!("Did not get any images from the API.")
        }
    }

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

trait Response {
    fn info_text(&self) -> anyhow::Result<Option<Vec<String>>>;
}

impl Response for Txt2ImgResponse {
    fn info_text(&self) -> anyhow::Result<Option<Vec<String>>> {
        Ok(self.info()?.infotexts)
    }
}

impl Response for Img2ImgResponse {
    fn info_text(&self) -> anyhow::Result<Option<Vec<String>>> {
        Ok(self.info()?.infotexts)
    }
}

fn message_from_resp<T: Response>(prompt: &str, resp: &T) -> anyhow::Result<String> {
    let mut message = format!("`{prompt}`");
    if let Some(infos) = resp.info_text()? {
        if let Some(info) = infos.get(0) {
            message = format!(
                "{message}\n{}",
                teloxide::utils::markdown::escape(info.strip_prefix(prompt).unwrap_or(info).trim())
            )
        }
    }
    Ok(message)
}

async fn handle_prompt(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (mut txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
    msg: Message,
    text: String,
) -> anyhow::Result<()> {
    let prompt = text;

    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    txt2img.prompt = Some(prompt.clone());

    let resp = cfg.api.txt2img()?.send(&txt2img).await?;

    use base64::{engine::general_purpose, Engine as _};

    let mut images: Vec<_> = resp
        .images
        .iter()
        .map(|i| {
            InputMedia::Photo(InputMediaPhoto::new(InputFile::memory(
                general_purpose::STANDARD
                    .decode(i)
                    .expect("failed to decode!"),
            )))
        })
        .collect();

    match images.len() {
        1 => {
            if let Some(image) = images.pop() {
                bot.send_photo(msg.chat.id, image.into())
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .caption(
                        message_from_resp(&prompt, &resp)
                            .context("Failed to build message from response")?,
                    )
                    .await?;
            }
        }
        2.. => {
            if let Some(InputMedia::Photo(image)) = images.get_mut(0) {
                image.caption = Some(
                    message_from_resp(&prompt, &resp)
                        .context("Failed to build message from response")?,
                );
            }
            bot.send_media_group(msg.chat.id, images).await?;
        }
        _ => {
            error!("Did not get any images from the API.")
        }
    }

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}
