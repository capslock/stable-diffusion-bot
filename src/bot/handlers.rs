use anyhow::{anyhow, Context};
use teloxide::{
    payloads::{setters::*, EditMessageReplyMarkupSetters},
    prelude::*,
    types::{
        ChatAction, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, InputMedia,
        InputMediaPhoto, MessageId, PhotoSize,
    },
    utils::command::BotCommands,
};
use tracing::error;

use crate::{
    api::{Img2ImgRequest, ImgResponse, Txt2ImgRequest},
    bot::{helpers::get_file, State},
};

use super::{ConfigParameters, DiffusionDialogue};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Simple commands")]
pub(crate) enum UnauthenticatedCommands {
    #[command(description = "shows this message.")]
    Help,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Authenticated commands")]
pub(crate) enum AuthenticatedCommands {
    #[command(description = "Set the value")]
    Set { value: u64 },
}

pub(crate) async fn unauthenticated_commands_handler(
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

async fn send_response(
    bot: &Bot,
    chat_id: ChatId,
    caption: String,
    images: Vec<String>,
    source: MessageId,
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
                    bot.send_photo(chat_id, image.media)
                        .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                        .caption(caption)
                        .reply_markup(keyboard())
                        .reply_to_message_id(source)
                        .await?;
                }
            }
        }
        2.. => {
            bot.send_media_group(chat_id, imgs)
                .reply_to_message_id(source)
                .await?;
            bot.send_message(
                chat_id,
                "What would you like to do? Select below, or enter a new prompt.",
            )
            .reply_markup(keyboard())
            .reply_to_message_id(source)
            .await?;
        }
        _ => {
            error!("Did not get any images from the API.")
        }
    }

    Ok(())
}

pub(crate) async fn handle_image(
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

    send_response(&bot, msg.chat.id, caption, resp.images, msg.id).await?;

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

pub(crate) async fn handle_prompt(
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

    send_response(&bot, msg.chat.id, caption, resp.images, msg.id).await?;

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

fn keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Rerun", "rerun"),
        InlineKeyboardButton::callback("Settings", "settings"),
    ]])
}

pub(crate) async fn handle_rerun(
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

    let id = message.id;
    let chat_id = message.chat.id;

    let parent = if let Some(parent) = message.reply_to_message().cloned() {
        parent
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Oops, something went wrong.")
            .await?;
        return Ok(());
    };

    if let Some(photo) = parent.photo().map(|p| p.to_vec()) {
        bot.answer_callback_query(q.id).await?;
        handle_image(
            bot.clone(),
            cfg,
            dialogue,
            (txt2img, img2img),
            parent,
            photo.to_vec(),
        )
        .await?;
    } else if let Some(caption) = parent.text() {
        bot.answer_callback_query(q.id).await?;
        let prompt = caption.to_owned();
        handle_prompt(
            bot.clone(),
            cfg,
            dialogue,
            (txt2img, img2img),
            parent,
            prompt,
        )
        .await?;
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Oops, something went wrong.")
            .await?;
        return Ok(());
    }

    bot.edit_message_reply_markup(chat_id, id)
        .reply_markup(InlineKeyboardMarkup::new([[]]))
        .send()
        .await?;

    Ok(())
}
