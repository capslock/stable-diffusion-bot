use anyhow::{anyhow, Context};
use sal_e_api::{GenParams, ImageParams, Response};
use teloxide::{
    dispatching::UpdateHandler,
    dptree::case,
    macros::BotCommands,
    payloads::setters::*,
    prelude::*,
    types::{
        ChatAction, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, InputMedia,
        InputMediaPhoto, Me, MessageId, PhotoSize,
    },
    utils::command::BotCommands as _,
};
use tracing::{info, instrument, warn};

use crate::{
    bot::{helpers, State},
    BotState,
};

use super::{
    filter_command, filter_map_bot_state, filter_map_settings, ConfigParameters, DiffusionDialogue,
};

/// BotCommands for generating images.
#[derive(BotCommands, Debug, Clone)]
#[command(rename_rule = "lowercase", description = "Image generation commands")]
pub(crate) enum GenCommands {
    /// Command to generate an image
    #[command(description = "generate an image")]
    Gen(String),
    /// Alias for `gen`. Hidden from help to avoid confusion.
    #[command(description = "off")]
    G(String),
    /// Alias for `gen`. Hidden from help to avoid confusion.
    #[command(description = "off")]
    Generate(String),
}

enum Photo {
    Single(Vec<u8>),
    Album(Vec<Vec<u8>>),
}

impl Photo {
    #[allow(dead_code)]
    pub fn single(photo: Vec<u8>) -> anyhow::Result<Self> {
        Ok(Self::Single(photo))
    }

    pub fn album(photos: Vec<Vec<u8>>) -> anyhow::Result<Self> {
        if photos.len() == 1 {
            let images = photos
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Failed to get image"))?;
            Ok(Photo::Single(images))
        } else {
            Ok(Photo::Album(photos))
        }
    }
}

struct ReplyConfig {
    disable_user_settings: bool,
    hide_all_buttons: bool,
    hide_settings_button: bool,
    hide_reuse_button: bool,
    hide_rerun_button: bool,
}

impl From<ConfigParameters> for ReplyConfig {
    fn from(config: ConfigParameters) -> Self {
        Self {
            disable_user_settings: config.settings.disable_user_settings,
            hide_all_buttons: config.ui.hide_all_buttons,
            hide_settings_button: config.ui.hide_settings_button,
            hide_reuse_button: config.ui.hide_reuse_button,
            hide_rerun_button: config.ui.hide_rerun_button,
        }
    }
}

struct Reply {
    caption: String,
    images: Photo,
    source: MessageId,
    seed: i64,
    config: ReplyConfig,
}

impl Reply {
    pub fn new(
        caption: String,
        images: Vec<Vec<u8>>,
        seed: i64,
        source: MessageId,
        config: ReplyConfig,
    ) -> anyhow::Result<Self> {
        let images = Photo::album(images)?;
        Ok(Self {
            caption,
            images,
            source,
            seed,
            config,
        })
    }

    pub async fn send(self, bot: &Bot, chat_id: ChatId) -> anyhow::Result<()> {
        match self.images {
            Photo::Single(image) => {
                let mut message = bot
                    .send_photo(chat_id, InputFile::memory(image))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .caption(self.caption)
                    .reply_to_message_id(self.source);
                if !self.config.hide_all_buttons {
                    message = message.reply_markup(keyboard(self.seed, self.config.into()))
                }
                message.await?;
            }
            Photo::Album(images) => {
                let mut caption = Some(self.caption);
                let input_media = images.into_iter().map(|i| {
                    let mut media = InputMediaPhoto::new(InputFile::memory(i));
                    media.caption = caption.take();
                    media.parse_mode = Some(teloxide::types::ParseMode::MarkdownV2);
                    InputMedia::Photo(media)
                });

                bot.send_media_group(chat_id, input_media)
                    .reply_to_message_id(self.source)
                    .await?;
                if !self.config.hide_all_buttons {
                    bot.send_message(
                        chat_id,
                        "What would you like to do? Select below, or enter a new prompt.",
                    )
                    .reply_markup(keyboard(self.seed, self.config.into()))
                    .reply_to_message_id(self.source)
                    .await?;
                }
            }
        }

        Ok(())
    }
}

struct MessageText(String);

impl MessageText {
    pub fn new(prompt: &str) -> Self {
        use teloxide::utils::markdown::escape;
        Self(format!("`{}`", escape(prompt)))
    }

    pub fn new_with_image_params(prompt: &str, infotxt: &dyn ImageParams) -> Self {
        use teloxide::utils::markdown::escape;

        Self(format!(
            "`{}`\n\n{}",
            escape(prompt),
            [
                infotxt
                    .negative_prompt()
                    .as_ref()
                    .and_then(|s| (!s.trim().is_empty()).then(|| escape(s)))
                    .map(|s| format!("Negative prompt: `{s}`")),
                infotxt.steps().map(|s| format!("Steps: `{s}`")),
                infotxt
                    .sampler()
                    .as_ref()
                    .map(|s| format!("Sampler: `{s}`")),
                infotxt.cfg().map(|s| format!("CFG scale: `{s}`")),
                infotxt.seed().map(|s| format!("Seed: `{s}`")),
                infotxt
                    .width()
                    .and_then(|w| infotxt.height().map(|h| format!("Size: `{w}×{h}`"))),
                infotxt.model().as_ref().map(|s| format!("Model: `{s}`")),
                infotxt
                    .denoising()
                    .map(|s| format!("Denoising strength: `{s}`")),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n")
        ))
    }
}

impl TryFrom<&dyn ImageParams> for MessageText {
    type Error = anyhow::Error;

    fn try_from(params: &dyn ImageParams) -> Result<Self, Self::Error> {
        let prompt = if let Some(prompt) = params.prompt() {
            prompt
        } else {
            return Err(anyhow!("No prompt in image info response"));
        };
        Ok(Self::new_with_image_params(prompt.as_str(), params))
    }
}

impl TryFrom<Response> for MessageText {
    type Error = anyhow::Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        Self::try_from(response.params.as_ref())
    }
}

async fn do_img2img(
    bot: &Bot,
    cfg: &ConfigParameters,
    img2img: &mut Box<dyn GenParams>,
    msg: &Message,
    photo: Vec<PhotoSize>,
    prompt: String,
) -> anyhow::Result<Response> {
    img2img.set_prompt(prompt);

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

    let photo = helpers::get_file(bot, &file).await?;

    img2img.set_image(Some(photo.into()));

    let resp = cfg.img2img_api.img2img(img2img.as_ref()).await?;

    img2img.set_image(None);

    Ok(resp)
}

async fn handle_image(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (txt2img, mut img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
    msg: Message,
    photo: Vec<PhotoSize>,
    text: String,
) -> anyhow::Result<()> {
    if text.is_empty() {
        bot.send_message(msg.chat.id, "A prompt is required.")
            .reply_to_message_id(msg.id)
            .await?;
        return Ok(());
    }

    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    let resp = do_img2img(&bot, &cfg, &mut img2img, &msg, photo, text).await?;

    let seed = if resp.params.seed() == resp.gen_params.seed() {
        -1
    } else {
        resp.params.seed().unwrap_or(-1)
    };

    let caption = if cfg.messages.hide_generation_info {
        MessageText::new(&resp.params.prompt().unwrap_or_default())
    } else {
        MessageText::try_from(resp.params.as_ref())
            .context("Failed to build caption from response")?
    };

    Reply::new(caption.0, resp.images, seed, msg.id, cfg.into())
        .context("Failed to create response!")?
        .send(&bot, msg.chat.id)
        .await?;

    dialogue
        .update(State::Ready {
            bot_state: BotState::default(),
            txt2img,
            img2img,
        })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

async fn do_txt2img(
    prompt: String,
    cfg: &ConfigParameters,
    txt2img: &mut dyn GenParams,
) -> anyhow::Result<Response> {
    txt2img.set_prompt(prompt);

    let resp = cfg.txt2img_api.txt2img(txt2img).await?;

    Ok(resp)
}

async fn handle_prompt(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (mut txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
    msg: Message,
    text: String,
) -> anyhow::Result<()> {
    if text.is_empty() {
        bot.send_message(msg.chat.id, "A prompt is required.")
            .reply_to_message_id(msg.id)
            .await?;
        return Ok(());
    }

    bot.send_chat_action(msg.chat.id, ChatAction::UploadPhoto)
        .await?;

    let resp = do_txt2img(text, &cfg, txt2img.as_mut()).await?;

    let seed = if resp.params.seed() == resp.gen_params.seed() {
        -1
    } else {
        resp.params.seed().unwrap_or(-1)
    };

    let caption = if cfg.messages.hide_generation_info {
        MessageText::new(&resp.params.prompt().unwrap_or_default())
    } else {
        MessageText::try_from(resp.params.as_ref())
            .context("Failed to build caption from response")?
    };

    Reply::new(caption.0, resp.images, seed, msg.id, cfg.into())
        .context("Failed to create response!")?
        .send(&bot, msg.chat.id)
        .await?;

    dialogue
        .update(State::Ready {
            bot_state: BotState::default(),
            txt2img,
            img2img,
        })
        .await
        .map_err(|e| anyhow!(e))?;

    Ok(())
}

struct KeyboardConfig {
    disable_user_settings: bool,
    hide_settings_button: bool,
    hide_reuse_button: bool,
    hide_rerun_button: bool,
}

impl From<ReplyConfig> for KeyboardConfig {
    fn from(config: ReplyConfig) -> Self {
        Self {
            disable_user_settings: config.disable_user_settings,
            hide_settings_button: config.hide_settings_button,
            hide_reuse_button: config.hide_reuse_button,
            hide_rerun_button: config.hide_rerun_button,
        }
    }
}

impl From<ConfigParameters> for KeyboardConfig {
    fn from(config: ConfigParameters) -> Self {
        Self {
            disable_user_settings: config.settings.disable_user_settings,
            hide_settings_button: config.ui.hide_settings_button,
            hide_reuse_button: config.ui.hide_reuse_button,
            hide_rerun_button: config.ui.hide_rerun_button,
        }
    }
}

fn keyboard(seed: i64, cfg: KeyboardConfig) -> InlineKeyboardMarkup {
    let settings_button = if (!cfg.disable_user_settings) && !cfg.hide_settings_button {
        vec![InlineKeyboardButton::callback("⚙️ Settings", "settings")]
    } else {
        vec![]
    };
    let seed_button = if cfg.hide_reuse_button {
        vec![]
    } else if seed == -1 {
        vec![InlineKeyboardButton::callback("🎲 Seed", "reuse/-1")]
    } else {
        vec![InlineKeyboardButton::callback(
            "♻️ Seed",
            format!("reuse/{seed}"),
        )]
    };
    let rerun_button = if cfg.hide_rerun_button {
        vec![]
    } else {
        vec![InlineKeyboardButton::callback("🔄 Rerun", "rerun")]
    };
    InlineKeyboardMarkup::new([[rerun_button, seed_button, settings_button].concat()])
}

#[instrument(skip_all)]
async fn handle_rerun(
    me: Me,
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
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

    if let Some(photo) = parent.photo().map(ToOwned::to_owned) {
        if let Some(text) = message.caption().map(ToOwned::to_owned) {
            let bot_name = me.user.username.expect("Bots must have a username");
            let text = if let Ok(command) = GenCommands::parse(&text, &bot_name) {
                match command {
                    GenCommands::Gen(s) | GenCommands::G(s) | GenCommands::Generate(s) => s,
                }
            } else {
                text
            };

            if let Err(e) = bot
                .answer_callback_query(q.id)
                .cache_time(60)
                .text("Rerunning this image...")
                .await
            {
                warn!("Failed to answer image rerun callback query: {}", e)
            }
            handle_image(
                bot.clone(),
                cfg,
                dialogue,
                (txt2img, img2img),
                parent,
                photo,
                text,
            )
            .await?;
        } else {
            bot.send_message(message.chat.id, "A prompt is required to run img2img.")
                .await?;
            return Err(anyhow!("No prompt provided for img2img"));
        }
    } else if let Some(text) = parent.text().map(ToOwned::to_owned) {
        if let Err(e) = bot
            .answer_callback_query(q.id)
            .cache_time(60)
            .text("Rerunning this prompt...")
            .await
        {
            warn!("Failed to answer prompt rerun callback query: {}", e)
        }
        let bot_name = me.user.username.expect("Bots must have a username");
        let text = if let Ok(command) = GenCommands::parse(&text, &bot_name) {
            match command {
                GenCommands::Gen(s) | GenCommands::G(s) | GenCommands::Generate(s) => s,
            }
        } else {
            text
        };
        handle_prompt(bot.clone(), cfg, dialogue, (txt2img, img2img), parent, text).await?;
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

async fn handle_reuse(
    bot: Bot,
    dialogue: DiffusionDialogue,
    (mut txt2img, mut img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
    q: CallbackQuery,
    seed: i64,
    cfg: ConfigParameters,
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

    if parent.photo().is_some() {
        img2img.set_seed(seed);
        dialogue
            .update(State::Ready {
                bot_state: BotState::default(),
                txt2img,
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;
    } else if parent.text().is_some() {
        txt2img.set_seed(seed);
        dialogue
            .update(State::Ready {
                bot_state: BotState::default(),
                txt2img,
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Oops, something went wrong.")
            .await?;
        return Ok(());
    }
    if seed == -1 {
        if let Err(e) = bot
            .answer_callback_query(q.id)
            .text("Seed randomized.")
            .await
        {
            warn!("Failed to answer randomize seed callback query: {}", e)
        }
    } else {
        if let Err(e) = bot
            .answer_callback_query(q.id)
            .text(format!("Seed set to {seed}."))
            .await
        {
            warn!("Failed to answer set seed callback query: {}", e)
        }
        bot.edit_message_reply_markup(chat_id, id)
            .reply_markup(keyboard(-1, cfg.into()))
            .send()
            .await?;
    }

    Ok(())
}

pub(crate) fn image_schema() -> UpdateHandler<anyhow::Error> {
    let gen_command_handler = Update::filter_message()
        .chain(filter_command::<GenCommands>())
        .chain(dptree::filter_map(|g: GenCommands| match g {
            GenCommands::Gen(s) | GenCommands::G(s) | GenCommands::Generate(s) => Some(s),
        }))
        .branch(Message::filter_photo().endpoint(handle_image))
        .branch(dptree::endpoint(handle_prompt));

    let message_handler = Update::filter_message()
        .branch(
            dptree::filter(|msg: Message| {
                msg.text().map(|t| t.starts_with('/')).unwrap_or_default()
            })
            .endpoint(|msg: Message| async move {
                info!(
                    "Ignoring unknown command: {}",
                    msg.text().unwrap_or_default()
                );
                Ok(())
            }),
        )
        .branch(
            Message::filter_photo()
                .map(|msg: Message| msg.caption().map(str::to_string).unwrap_or_default())
                .endpoint(handle_image),
        )
        .branch(Message::filter_text().endpoint(handle_prompt));

    let callback_handler = Update::filter_callback_query()
        .branch(
            dptree::filter_map(|q: CallbackQuery| {
                q.data
                    .filter(|d| d.starts_with("reuse"))
                    .and_then(|d| d.split('/').skip(1).flat_map(str::parse::<i64>).next())
            })
            .endpoint(handle_reuse),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| q.data.filter(|d| d.starts_with("rerun")).is_some())
                .endpoint(handle_rerun),
        );

    dptree::entry()
        .chain(filter_map_bot_state())
        .chain(case![BotState::Generate])
        .chain(filter_map_settings())
        .branch(gen_command_handler)
        .branch(message_handler)
        .branch(callback_handler)
}
