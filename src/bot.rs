use std::collections::HashSet;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::{
        dialogue::{self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage},
        UpdateHandler,
    },
    dptree::case,
    payloads::{setters::*, EditMessageReplyMarkupSetters},
    prelude::*,
    types::{
        ChatAction, File, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, InputMedia,
        InputMediaPhoto, MessageId, PhotoSize, Update, UserId,
    },
    utils::command::BotCommands,
};
use tracing::{error, warn};

use crate::api::{Api, Img2ImgRequest, ImgResponse, Txt2ImgRequest};

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

async fn get_file(client: reqwest::Client, bot: &Bot, file: &File) -> anyhow::Result<bytes::Bytes> {
    client
        .get(format!(
            "https://api.telegram.org/file/bot{}/{}",
            bot.token(),
            file.path
        ))
        .send()
        .await
        .context("Failed to get file")?
        .bytes()
        .await
        .context("Failed to get bytes")
}

enum Source {
    Photo(MessageId),
    Text(MessageId),
}

async fn send_response(
    bot: &Bot,
    chat_id: ChatId,
    caption: String,
    images: Vec<String>,
    source: Source,
) -> anyhow::Result<()> {
    use base64::{engine::general_purpose, Engine as _};

    let mut caption = Some(caption);

    let mut imgs = images
        .iter()
        .map(|i| {
            let mut photo = InputMediaPhoto::new(InputFile::memory(
                general_purpose::STANDARD
                    .decode(i)
                    .context("Failed to decode image")?,
            ))
            .parse_mode(teloxide::types::ParseMode::MarkdownV2);
            photo.caption = caption.take();
            Ok(InputMedia::Photo(photo))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    match imgs.len() {
        1 => {
            if let Some(InputMedia::Photo(image)) = imgs.pop() {
                if let Some(caption) = image.caption {
                    let req = bot
                        .send_photo(chat_id, image.media)
                        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                        .caption(caption);
                    match source {
                        Source::Photo(source) => {
                            req.reply_to_message_id(source).reply_markup(keyboard_img())
                        }
                        Source::Text(source) => {
                            req.reply_to_message_id(source).reply_markup(keyboard_txt())
                        }
                    }
                    .await?;
                }
            }
        }
        2.. => {
            let req = bot.send_media_group(chat_id, imgs);
            match source {
                Source::Photo(source) | Source::Text(source) => req.reply_to_message_id(source),
            }
            .await?;
            let req = bot.send_message(
                chat_id,
                "What would you like to do? Select below, or enter a new prompt.",
            );
            match source {
                Source::Photo(source) => {
                    req.reply_to_message_id(source).reply_markup(keyboard_img())
                }
                Source::Text(source) => {
                    req.reply_to_message_id(source).reply_markup(keyboard_txt())
                }
            }
            .await?;
        }
        _ => {
            error!("Did not get any images from the API.")
        }
    }

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

    let photo = get_file(cfg.client, &bot, &file).await?;

    use base64::{engine::general_purpose, Engine as _};

    let photo = general_purpose::STANDARD.encode(photo);

    img2img.init_images = Some(vec![photo]);

    let resp = cfg.api.img2img()?.send(&img2img).await?;

    let caption =
        message_from_resp(prompt, &resp).context("Failed to build caption from response")?;

    send_response(
        &bot,
        msg.chat.id,
        caption,
        resp.images,
        Source::Photo(msg.id),
    )
    .await?;

    _ = img2img.init_images.take();

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

fn message_from_resp<T>(prompt: &str, resp: &ImgResponse<T>) -> anyhow::Result<String> {
    let mut message = format!("`{prompt}`");
    if let Some(infos) = resp.info()?.infotexts {
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

    let caption =
        message_from_resp(&prompt, &resp).context("Failed to build caption from response")?;

    send_response(
        &bot,
        msg.chat.id,
        caption,
        resp.images,
        Source::Text(msg.id),
    )
    .await?;

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

fn keyboard_img() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Rerun", "rerun-img"),
        InlineKeyboardButton::callback("Settings", "settings"),
    ]])
}

fn keyboard_txt() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Rerun", "rerun-txt"),
        InlineKeyboardButton::callback("Settings", "settings"),
    ]])
}

async fn handle_rerun(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
    q: CallbackQuery,
) -> anyhow::Result<()> {
    let message = if let Some(message) = q.message {
        message
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Sorry, this message is no longer available.")
            .await?;
        return Ok(());
    };

    let data = if let Some(data) = q.data {
        data
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Something went wrong.")
            .await?;
        return Ok(());
    };

    let id = message.id;
    let chat_id = message.chat.id;

    match data.as_str() {
        "rerun-txt" => {
            let message = message.reply_to_message().cloned().unwrap_or(message);
            bot.answer_callback_query(q.id).await?;
            if let Some(caption) = message.text() {
                let prompt = caption.to_owned();
                handle_prompt(
                    bot.clone(),
                    cfg,
                    dialogue,
                    (txt2img, img2img),
                    message,
                    prompt,
                )
                .await?;
            }
        }
        "rerun-img" => {
            bot.answer_callback_query(q.id).await?;
            let parent = message.reply_to_message().cloned().unwrap_or(message);
            if let Some(photo) = parent.photo().map(|p| p.to_vec()) {
                handle_image(
                    bot.clone(),
                    cfg,
                    dialogue,
                    (txt2img, img2img),
                    parent,
                    photo.to_vec(),
                )
                .await?;
            }
        }
        _ => {
            bot.answer_callback_query(q.id)
                .cache_time(60)
                .text("Oops, something went wrong.")
                .await?;
            return Ok(());
        }
    }
    bot.edit_message_reply_markup(chat_id, id)
        .reply_markup(InlineKeyboardMarkup::new([[]]))
        .send()
        .await?;

    Ok(())
}
