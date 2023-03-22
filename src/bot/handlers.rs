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

use crate::{
    api::{Img2ImgRequest, ImgResponse, Txt2ImgRequest},
    bot::{helpers, State},
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

enum Photo {
    Single(Vec<u8>),
    Album(Vec<Vec<u8>>),
}

struct Response {
    caption: String,
    images: Photo,
    source: MessageId,
}

impl Response {
    pub fn new(caption: String, images: Vec<String>, source: MessageId) -> anyhow::Result<Self> {
        use base64::{engine::general_purpose, Engine as _};
        let images = images
            .into_iter()
            .map(|i| {
                general_purpose::STANDARD
                    .decode(i)
                    .context("Failed to decode image")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;
        let images = match images.len() {
            1 => Photo::Single(
                images
                    .into_iter()
                    .next()
                    .ok_or(anyhow!("Failed to get image"))?,
            ),
            2.. => Photo::Album(images),
            _ => return Err(anyhow!("Must provide at least one image!")),
        };
        Ok(Self {
            caption,
            images,
            source,
        })
    }

    pub async fn send(self, bot: &Bot, chat_id: ChatId) -> anyhow::Result<()> {
        match self.images {
            Photo::Single(image) => {
                bot.send_photo(chat_id, InputFile::memory(image))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .caption(self.caption)
                    .reply_markup(keyboard())
                    .reply_to_message_id(self.source)
                    .await?;
            }
            Photo::Album(images) => {
                let mut caption = Some(self.caption);
                let input_media = images.into_iter().map(|i| {
                    let mut media = InputMediaPhoto::new(InputFile::memory(i));
                    media.caption = caption.take();
                    InputMedia::Photo(media)
                });

                bot.send_media_group(chat_id, input_media)
                    .reply_to_message_id(self.source)
                    .await?;
                bot.send_message(
                    chat_id,
                    "What would you like to do? Select below, or enter a new prompt.",
                )
                .reply_markup(keyboard())
                .reply_to_message_id(self.source)
                .await?;
            }
        }

        Ok(())
    }
}

struct MessageText(String);

impl MessageText {
    pub fn new_with_infotxt(prompt: &str, infotxt: &str) -> Self {
        use teloxide::utils::markdown::escape;
        Self(format!(
            "`{}`\n{}",
            escape(prompt),
            escape(infotxt.strip_prefix(prompt).unwrap_or(infotxt).trim())
        ))
    }

    pub fn new(prompt: &str) -> Self {
        use teloxide::utils::markdown::escape;
        Self(format!("`{}`", escape(prompt),))
    }
}

impl<T> TryFrom<ImgResponse<T>> for MessageText {
    type Error = anyhow::Error;

    fn try_from(resp: ImgResponse<T>) -> Result<Self, Self::Error> {
        resp.try_into()
    }
}

impl<T> TryFrom<&ImgResponse<T>> for MessageText {
    type Error = anyhow::Error;

    fn try_from(resp: &ImgResponse<T>) -> Result<Self, Self::Error> {
        let info = resp.info()?;
        let prompt = if let Some(prompt) = info.prompt {
            prompt
        } else {
            return Err(anyhow!("No prompt in image info response"));
        };
        if let Some(infos) = info.infotexts {
            if let Some(info) = infos.get(0) {
                return Ok(Self::new_with_infotxt(prompt.as_str(), info.as_str()));
            }
        }
        return Ok(Self::new(prompt.as_str()));
    }
}

async fn do_img2img(
    bot: &Bot,
    cfg: &ConfigParameters,
    img2img: &mut Img2ImgRequest,
    msg: &Message,
    photo: Vec<PhotoSize>,
) -> anyhow::Result<ImgResponse<Img2ImgRequest>> {
    let prompt = if let Some(caption) = msg.caption() {
        caption
    } else {
        bot.send_message(msg.chat.id, "A prompt is required to run img2img.")
            .await?;
        return Err(anyhow!("No prompt provided for img2img"));
    };

    img2img.prompt = Some(prompt.to_owned());

    let photo = if let Some(photo) = photo
        .iter()
        .reduce(|a, p| if a.height > p.height { a } else { p })
    {
        photo
    } else {
        bot.send_message(msg.chat.id, "Something went wrong.")
            .await?;
        return Err(anyhow!("Photo vec was empty!"));
    };
    let file = bot.get_file(&photo.file.id).send().await?;

    let photo = helpers::get_file(&cfg.client, bot, &file).await?;

    img2img.with_image(photo);

    let resp = cfg.api.img2img()?.send(img2img).await?;

    _ = img2img.init_images.take();

    Ok(resp)
}

pub(crate) async fn handle_image(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (txt2img, mut img2img): (Txt2ImgRequest, Img2ImgRequest),
    msg: Message,
    photo: Vec<PhotoSize>,
) -> anyhow::Result<()> {
    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    let resp = do_img2img(&bot, &cfg, &mut img2img, &msg, photo).await?;

    let caption = MessageText::try_from(&resp).context("Failed to build caption from response")?;
    Response::new(caption.0, resp.images, msg.id)
        .context("Failed to create response!")?
        .send(&bot, msg.chat.id)
        .await?;

    dialogue
        .update(State::Ready { txt2img, img2img })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

async fn do_txt2img(
    prompt: &str,
    cfg: &ConfigParameters,
    txt2img: &mut Txt2ImgRequest,
) -> anyhow::Result<ImgResponse<Txt2ImgRequest>> {
    txt2img.with_prompt(prompt.to_owned());

    let resp = cfg.api.txt2img()?.send(txt2img).await?;

    Ok(resp)
}

pub(crate) async fn handle_prompt(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (mut txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
    msg: Message,
    text: String,
) -> anyhow::Result<()> {
    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    let resp = do_txt2img(text.as_str(), &cfg, &mut txt2img).await?;

    let caption = MessageText::try_from(&resp).context("Failed to build caption from response")?;
    Response::new(caption.0, resp.images, msg.id)
        .context("Failed to create response!")?
        .send(&bot, msg.chat.id)
        .await?;

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
