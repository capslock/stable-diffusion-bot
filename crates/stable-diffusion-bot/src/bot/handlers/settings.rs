use anyhow::anyhow;
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
use teloxide::{
    dispatching::UpdateHandler,
    dptree::case,
    macros::BotCommands,
    payloads::setters::*,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};
use tracing::error;

use crate::{bot::ConfigParameters, BotState};

use super::{filter_map_bot_state, filter_map_settings, DiffusionDialogue, State};

/// BotCommands for settings.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Authenticated commands")]
pub(crate) enum SettingsCommands {
    /// Command to set txt2img settings
    #[command(description = "txt2img settings")]
    Txt2ImgSettings,
    /// Command to set img2img settings
    #[command(description = "img2img settings")]
    Img2ImgSettings,
}

/// User-configurable image generation settings.
#[allow(dead_code)]
pub(crate) struct Settings {
    // Number of sampling steps.
    pub steps: u32,
    // Random seed.
    pub seed: i64,
    // Number of images to generate per batch.
    pub batch_size: u32,
    // Number of batches of images to generate.
    pub n_iter: u32,
    // CFG scale.
    pub cfg_scale: f64,
    // Image width.
    pub width: u32,
    // Image height.
    pub height: u32,
    // Negative prompt.
    pub negative_prompt: String,
    // Styles to apply.
    pub styles: Vec<String>,
    // Whether to run a restore faces pass.
    pub restore_faces: bool,
    // Enable or disable image tiling.
    pub tiling: bool,
    // Denoising strength. Only used for img2img.
    pub denoising_strength: Option<f64>,
    // Sampler name.
    pub sampler_index: String,
}

impl Settings {
    /// Build an inline keyboard to configure settings.
    pub fn keyboard(&self) -> InlineKeyboardMarkup {
        let keyboard = InlineKeyboardMarkup::new([
            [
                InlineKeyboardButton::callback(format!("Steps: {}", self.steps), "settings_steps"),
                InlineKeyboardButton::callback(format!("Seed: {}", self.seed), "settings_seed"),
            ],
            [
                InlineKeyboardButton::callback(
                    format!("Batch Count: {}", self.n_iter),
                    "settings_count",
                ),
                InlineKeyboardButton::callback(
                    format!("CFG Scale: {}", self.cfg_scale),
                    "settings_cfg",
                ),
            ],
            [
                InlineKeyboardButton::callback(format!("Width: {}", self.width), "settings_width"),
                InlineKeyboardButton::callback(
                    format!("Height: {}", self.height),
                    "settings_height",
                ),
            ],
            [
                InlineKeyboardButton::callback("Negative Prompt".to_owned(), "settings_negative"),
                InlineKeyboardButton::callback("Styles".to_owned(), "settings_styles"),
            ],
            [
                InlineKeyboardButton::callback(
                    format!("Tiling: {}", self.tiling),
                    "settings_tiling",
                ),
                InlineKeyboardButton::callback(
                    format!("Restore Faces: {}", self.restore_faces),
                    "settings_faces",
                ),
            ],
        ]);

        if let Some(d) = self.denoising_strength {
            keyboard.append_row([
                InlineKeyboardButton::callback(
                    format!("Denoising Strength: {d}"),
                    "settings_denoising",
                ),
                InlineKeyboardButton::callback("Cancel".to_owned(), "settings_back"),
            ])
        } else {
            keyboard.append_row([InlineKeyboardButton::callback(
                "Cancel".to_owned(),
                "settings_back",
            )])
        }
    }
}

impl TryFrom<&Txt2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: &Txt2ImgRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            steps: value.steps.ok_or_else(|| anyhow!("Missing steps!"))?,
            seed: value.seed.ok_or_else(|| anyhow!("Missing seed!"))?,
            batch_size: value
                .batch_size
                .ok_or_else(|| anyhow!("Missing batch_size!"))?,
            n_iter: value.n_iter.ok_or_else(|| anyhow!("Missing n_iter!"))?,
            cfg_scale: value
                .cfg_scale
                .ok_or_else(|| anyhow!("Missing cfg_scale!"))?,
            width: value.width.ok_or_else(|| anyhow!("Missing width!"))?,
            height: value.height.ok_or_else(|| anyhow!("Missing height!"))?,
            negative_prompt: value
                .negative_prompt
                .clone()
                .ok_or_else(|| anyhow!("Missing negative_prompt!"))?,
            styles: value
                .styles
                .clone()
                .ok_or_else(|| anyhow!("Missing styles!"))?,
            restore_faces: value
                .restore_faces
                .ok_or_else(|| anyhow!("Missing restore_faces!"))?,
            tiling: value.tiling.ok_or_else(|| anyhow!("Missing tiling!"))?,
            denoising_strength: value.denoising_strength,
            sampler_index: value
                .sampler_index
                .clone()
                .ok_or_else(|| anyhow!("Missing sampler_index!"))?,
        })
    }
}

impl TryFrom<Txt2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: Txt2ImgRequest) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&Img2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: &Img2ImgRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            steps: value.steps.ok_or_else(|| anyhow!("Missing steps!"))?,
            seed: value.seed.ok_or_else(|| anyhow!("Missing seed!"))?,
            batch_size: value
                .batch_size
                .ok_or_else(|| anyhow!("Missing batch_size!"))?,
            n_iter: value.n_iter.ok_or_else(|| anyhow!("Missing n_iter!"))?,
            cfg_scale: value
                .cfg_scale
                .ok_or_else(|| anyhow!("Missing cfg_scale!"))?,
            width: value.width.ok_or_else(|| anyhow!("Missing width!"))?,
            height: value.height.ok_or_else(|| anyhow!("Missing height!"))?,
            negative_prompt: value
                .negative_prompt
                .clone()
                .ok_or_else(|| anyhow!("Missing negative_prompt!"))?,
            styles: value
                .styles
                .clone()
                .ok_or_else(|| anyhow!("Missing styles!"))?,
            restore_faces: value
                .restore_faces
                .ok_or_else(|| anyhow!("Missing restore_faces!"))?,
            tiling: value.tiling.ok_or_else(|| anyhow!("Missing tiling!"))?,
            denoising_strength: value.denoising_strength,
            sampler_index: value
                .sampler_index
                .clone()
                .ok_or_else(|| anyhow!("Missing sampler_index!"))?,
        })
    }
}

impl TryFrom<Img2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: Img2ImgRequest) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

pub(crate) fn filter_callback_query_chat_id() -> UpdateHandler<anyhow::Error> {
    dptree::filter_map(|q: CallbackQuery| q.message.map(|m| m.chat.id))
}

pub(crate) async fn handle_message_expired(bot: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id)
        .cache_time(60)
        .text("Sorry, this message is no longer available.")
        .await?;
    Ok(())
}

pub(crate) fn filter_callback_query_parent() -> UpdateHandler<anyhow::Error> {
    dptree::filter_map(|q: CallbackQuery| q.message.and_then(|m| m.reply_to_message().cloned()))
}

pub(crate) async fn handle_parent_unavailable(bot: Bot, q: CallbackQuery) -> anyhow::Result<()> {
    bot.answer_callback_query(q.id)
        .cache_time(60)
        .text("Oops, something went wrong.")
        .await?;
    Ok(())
}

pub(crate) async fn handle_settings(
    bot: Bot,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
    q: CallbackQuery,
    chat_id: ChatId,
    parent: Message,
) -> anyhow::Result<()> {
    let settings = if parent.photo().is_some() {
        let settings = Settings::try_from(&img2img)?;
        dialogue
            .update(State::Ready {
                bot_state: BotState::SettingsImg2Img { selection: None },
                txt2img,
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;
        settings
    } else if parent.text().is_some() {
        let settings = Settings::try_from(&txt2img)?;
        dialogue
            .update(State::Ready {
                bot_state: BotState::SettingsTxt2Img { selection: None },
                txt2img: txt2img.clone(),
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;
        settings
    } else {
        bot.answer_callback_query(q.id)
            .cache_time(60)
            .text("Oops, something went wrong.")
            .await?;
        return Ok(());
    };

    bot.send_message(chat_id, "Please make a selection.")
        .reply_markup(settings.keyboard())
        .send()
        .await?;

    Ok(())
}

pub(crate) async fn handle_settings_button(
    bot: Bot,
    cfg: ConfigParameters,
    dialogue: DiffusionDialogue,
    (_, txt2img, img2img): (Option<String>, Txt2ImgRequest, Img2ImgRequest),
    q: CallbackQuery,
) -> anyhow::Result<()> {
    let (message, data) = match q {
        CallbackQuery {
            message: Some(message),
            data: Some(data),
            ..
        } => (message, data),
        _ => {
            bot.answer_callback_query(q.id)
                .cache_time(60)
                .text("Sorry, something went wrong.")
                .await?;
            return Ok(());
        }
    };

    let setting = match data.strip_prefix("settings_") {
        Some(setting) => setting,
        None => {
            bot.answer_callback_query(q.id)
                .cache_time(60)
                .text("Sorry, something went wrong.")
                .await?;
            return Ok(());
        }
    };

    if setting == "back" {
        dialogue
            .update(State::Ready {
                bot_state: BotState::Generate,
                txt2img,
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;
        bot.answer_callback_query(q.id).text("Canceled.").await?;
        bot.edit_message_text(message.chat.id, message.id, "Please enter a prompt.")
            .reply_markup(InlineKeyboardMarkup::new([[]]))
            .await?;
        return Ok(());
    }

    let mut state = dialogue
        .get()
        .await
        .map_err(|e| anyhow!(e))?
        .unwrap_or_else(|| State::new_with_defaults(cfg.txt2img_defaults, cfg.img2img_defaults));
    match &mut state {
        State::Ready {
            bot_state: BotState::SettingsTxt2Img { selection },
            ..
        }
        | State::Ready {
            bot_state: BotState::SettingsImg2Img { selection },
            ..
        } => *selection = Some(setting.to_string()),
        _ => {
            bot.answer_callback_query(q.id)
                .cache_time(60)
                .text("Sorry, something went wrong.")
                .await?;
            return Ok(());
        }
    }

    bot.answer_callback_query(q.id).await?;
    dialogue.update(state).await.map_err(|e| anyhow!(e))?;

    bot.send_message(message.chat.id, "Please enter a new value.")
        .await?;

    Ok(())
}

fn update_txt2img_setting<S1, S2>(
    txt2img: &mut Txt2ImgRequest,
    setting: S1,
    value: S2,
) -> anyhow::Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let value = value.as_ref();
    match setting.as_ref() {
        "steps" => txt2img.steps = Some(value.parse()?),
        "seed" => txt2img.seed = Some(value.parse()?),
        "count" => txt2img.n_iter = Some(value.parse()?),
        "cfg" => txt2img.cfg_scale = Some(value.parse()?),
        "width" => txt2img.width = Some(value.parse()?),
        "height" => txt2img.height = Some(value.parse()?),
        "negative" => txt2img.negative_prompt = Some(value.to_owned()),
        "styles" => txt2img.styles = Some(value.split(' ').map(ToOwned::to_owned).collect()),
        "tiling" => txt2img.tiling = Some(value.parse()?),
        "faces" => txt2img.restore_faces = Some(value.parse()?),
        "denoising" => txt2img.denoising_strength = Some(value.parse()?),
        _ => return Err(anyhow!("Got invalid setting: {}", setting.as_ref())),
    }
    Ok(())
}

fn update_img2img_setting<S1, S2>(
    img2img: &mut Img2ImgRequest,
    setting: S1,
    value: S2,
) -> anyhow::Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let value = value.as_ref();
    match setting.as_ref() {
        "steps" => img2img.steps = Some(200.min(value.parse()?)),
        "seed" => img2img.seed = Some((-1).max(value.parse()?)),
        "count" => img2img.n_iter = Some(value.parse::<u32>()?.clamp(1, 10)),
        "cfg" => img2img.cfg_scale = Some(value.parse::<f64>()?.clamp(0.0, 20.0)),
        "width" => {
            img2img.width = {
                let mut value = value.parse::<u32>()?;
                value -= value % 64;
                Some(value.clamp(64, 1024))
            }
        }
        "height" => {
            img2img.height = {
                let mut value = value.parse::<u32>()?;
                value -= value % 64;
                Some(value.clamp(64, 1024))
            }
        }
        "negative" => img2img.negative_prompt = Some(value.to_owned()),
        "styles" => img2img.styles = Some(value.split(' ').map(ToOwned::to_owned).collect()),
        "tiling" => img2img.tiling = Some(value.parse()?),
        "faces" => img2img.restore_faces = Some(value.parse()?),
        "denoising" => img2img.denoising_strength = Some(value.parse::<f64>()?.clamp(0.0, 1.0)),
        _ => return Err(anyhow!("invalid setting: {}", setting.as_ref())),
    }
    Ok(())
}

pub(crate) fn state_or_default() -> UpdateHandler<anyhow::Error> {
    dptree::map_async(
        |cfg: ConfigParameters, dialogue: DiffusionDialogue| async move {
            let result = dialogue.get().await;
            if let Err(ref err) = result {
                error!("Failed to get state: {:?}", err);
            }
            result.ok().flatten().unwrap_or_else(|| {
                State::new_with_defaults(cfg.txt2img_defaults, cfg.img2img_defaults)
            })
        },
    )
}

pub(crate) async fn update_settings_value(
    bot: Bot,
    dialogue: DiffusionDialogue,
    chat_id: ChatId,
    settings: Settings,
    state: State,
) -> anyhow::Result<()> {
    dialogue.update(state).await.map_err(|e| anyhow!(e))?;

    bot.send_message(chat_id, "Please make a selection.")
        .reply_markup(settings.keyboard())
        .await?;

    Ok(())
}

pub(crate) async fn handle_txt2img_settings_value(
    bot: Bot,
    dialogue: DiffusionDialogue,
    msg: Message,
    text: String,
    (selection, mut txt2img, img2img): (Option<String>, Txt2ImgRequest, Img2ImgRequest),
) -> anyhow::Result<()> {
    if let Some(ref setting) = selection {
        if let Err(e) = update_txt2img_setting(&mut txt2img, setting, text) {
            bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                .await?;
            return Ok(());
        }
    }

    let bot_state = BotState::SettingsTxt2Img { selection };

    update_settings_value(
        bot,
        dialogue,
        msg.chat.id,
        Settings::try_from(&txt2img)?,
        State::Ready {
            bot_state,
            txt2img,
            img2img,
        },
    )
    .await
}

pub(crate) async fn handle_img2img_settings_value(
    bot: Bot,
    dialogue: DiffusionDialogue,
    msg: Message,
    text: String,
    (selection, txt2img, mut img2img): (Option<String>, Txt2ImgRequest, Img2ImgRequest),
) -> anyhow::Result<()> {
    if let Some(ref setting) = selection {
        if let Err(e) = update_img2img_setting(&mut img2img, setting, text) {
            bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                .await?;
            return Ok(());
        }
    }

    let bot_state = BotState::SettingsImg2Img { selection };

    update_settings_value(
        bot,
        dialogue,
        msg.chat.id,
        Settings::try_from(&img2img)?,
        State::Ready {
            bot_state,
            txt2img,
            img2img,
        },
    )
    .await
}

pub(crate) fn map_settings() -> UpdateHandler<anyhow::Error> {
    dptree::map(|cfg: ConfigParameters, state: State| match state {
        State::Ready {
            txt2img, img2img, ..
        } => (txt2img, img2img),
        State::New => (cfg.txt2img_defaults, cfg.img2img_defaults),
    })
}

async fn handle_img2img_settings_command(
    msg: Message,
    bot: Bot,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
) -> anyhow::Result<()> {
    let settings = Settings::try_from(&img2img)?;
    dialogue
        .update(State::Ready {
            bot_state: BotState::SettingsImg2Img { selection: None },
            txt2img,
            img2img,
        })
        .await
        .map_err(|e| anyhow!(e))?;
    bot.send_message(msg.chat.id, "Please make a selection.")
        .reply_markup(settings.keyboard())
        .send()
        .await?;
    Ok(())
}

async fn handle_txt2img_settings_command(
    msg: Message,
    bot: Bot,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest),
) -> anyhow::Result<()> {
    let settings = Settings::try_from(&txt2img)?;
    dialogue
        .update(State::Ready {
            bot_state: BotState::SettingsTxt2Img { selection: None },
            txt2img,
            img2img,
        })
        .await
        .map_err(|e| anyhow!(e))?;
    bot.send_message(msg.chat.id, "Please make a selection.")
        .reply_markup(settings.keyboard())
        .send()
        .await?;
    Ok(())
}

async fn handle_invalid_setting_value(bot: Bot, msg: Message) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Please enter a valid value.")
        .await?;
    Ok(())
}

pub(crate) fn settings_command_handler() -> UpdateHandler<anyhow::Error> {
    Update::filter_message()
        .filter_command::<SettingsCommands>()
        .chain(state_or_default())
        .chain(map_settings())
        .branch(case![SettingsCommands::Txt2ImgSettings].endpoint(handle_txt2img_settings_command))
        .branch(case![SettingsCommands::Img2ImgSettings].endpoint(handle_img2img_settings_command))
}

pub(crate) fn filter_settings_callback_query() -> UpdateHandler<anyhow::Error> {
    Update::filter_callback_query()
        .filter(|q: CallbackQuery| q.data.is_some_and(|data| data.starts_with("settings")))
}

pub(crate) fn filter_settings_state() -> UpdateHandler<anyhow::Error> {
    dptree::filter(|state: State| {
        let bot_state = match state {
            State::Ready { bot_state, .. } => bot_state,
            _ => return false,
        };
        matches!(
            bot_state,
            BotState::SettingsTxt2Img { .. } | BotState::SettingsImg2Img { .. }
        )
    })
}

pub(crate) fn filter_map_settings_state() -> UpdateHandler<anyhow::Error> {
    dptree::filter_map(|state: State| {
        let (bot_state, txt2img, img2img) = match state {
            State::Ready {
                bot_state,
                txt2img,
                img2img,
            } => (bot_state, txt2img, img2img),
            _ => return None,
        };
        match bot_state {
            BotState::SettingsTxt2Img { selection } => Some((selection, txt2img, img2img)),
            BotState::SettingsImg2Img { selection } => Some((selection, txt2img, img2img)),
            _ => None,
        }
    })
}

pub(crate) fn settings_schema() -> UpdateHandler<anyhow::Error> {
    let callback_handler = filter_settings_callback_query()
        .branch(
            filter_map_bot_state()
                .chain(case![BotState::Generate])
                .chain(filter_map_settings())
                .branch(
                    filter_callback_query_chat_id()
                        .branch(filter_callback_query_parent().endpoint(handle_settings))
                        .endpoint(handle_parent_unavailable),
                )
                .endpoint(handle_message_expired),
        )
        .branch(filter_map_settings_state().endpoint(handle_settings_button));

    let message_handler = Update::filter_message()
        .branch(
            Message::filter_text()
                .chain(filter_map_settings_state())
                .chain(state_or_default())
                .chain(filter_map_bot_state())
                .branch(
                    case![BotState::SettingsTxt2Img { selection }]
                        .endpoint(handle_txt2img_settings_value),
                )
                .branch(
                    case![BotState::SettingsImg2Img { selection }]
                        .endpoint(handle_img2img_settings_value),
                )
                .endpoint(|| async { Err(anyhow!("Invalid settings state")) }),
        )
        .branch(filter_settings_state().endpoint(handle_invalid_setting_value));

    dptree::entry()
        .branch(settings_command_handler())
        .branch(message_handler)
        .branch(callback_handler)
}

#[cfg(test)]
mod tests {
    use teloxide::types::{UpdateKind, User};

    use super::*;
    use crate::BotState;

    fn create_callback_query_update(data: Option<String>) -> Update {
        let query = CallbackQuery {
            id: "123456".to_string(),
            from: User {
                id: UserId(123456780),
                is_bot: true,
                first_name: "Stable Diffusion".to_string(),
                last_name: None,
                username: Some("sdbot".to_string()),
                language_code: Some("en".to_string()),
                is_premium: false,
                added_to_attachment_menu: false,
            },
            message: None,
            inline_message_id: None,
            chat_instance: "123456".to_string(),
            data,
            game_short_name: None,
        };

        Update {
            id: 1,
            kind: UpdateKind::CallbackQuery(query),
        }
    }

    #[tokio::test]
    async fn test_filter_settings_query() {
        let update = create_callback_query_update(Some("settings".to_string()));

        assert!(matches!(
            filter_settings_callback_query()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![update])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_settings_query_none() {
        let update = create_callback_query_update(None);

        assert!(matches!(
            filter_settings_callback_query()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![update])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_settings_query_bad_data() {
        let update = create_callback_query_update(Some("bad_data".to_string()));

        assert!(matches!(
            filter_settings_callback_query()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![update])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_settings_state_txt2img() {
        assert!(matches!(
            filter_settings_state()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsTxt2Img { selection: None },
                    txt2img: Txt2ImgRequest::default(),
                    img2img: Img2ImgRequest::default()
                }])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_settings_state_img2img() {
        assert!(matches!(
            filter_settings_state()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsImg2Img { selection: None },
                    txt2img: Txt2ImgRequest::default(),
                    img2img: Img2ImgRequest::default()
                }])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_settings_state() {
        assert!(matches!(
            filter_settings_state()
                .endpoint(|| async { anyhow::Ok(()) })
                .dispatch(dptree::deps![State::New])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_map_settings_state_txt2img() {
        assert!(matches!(
            filter_map_settings_state()
                .endpoint(
                    |(_, _, _): (Option<String>, Txt2ImgRequest, Img2ImgRequest)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsTxt2Img { selection: None },
                    txt2img: Txt2ImgRequest::default(),
                    img2img: Img2ImgRequest::default()
                }])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_map_settings_state_img2img() {
        assert!(matches!(
            filter_map_settings_state()
                .endpoint(
                    |(_, _, _): (Option<String>, Txt2ImgRequest, Img2ImgRequest)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsImg2Img { selection: None },
                    txt2img: Txt2ImgRequest::default(),
                    img2img: Img2ImgRequest::default()
                }])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_map_settings_state() {
        assert!(matches!(
            filter_map_settings_state()
                .endpoint(
                    |(_, _, _): (Option<String>, Txt2ImgRequest, Img2ImgRequest)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::New])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[tokio::test]
    async fn test_map_settings_default() {
        assert!(matches!(
            map_settings()
                .endpoint(
                    |(txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest)| async {
                        assert!(
                            (txt2img, img2img)
                                == (Txt2ImgRequest::default(), Img2ImgRequest::default())
                        );
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![ConfigParameters::default(), State::New])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_map_settings_ready() {
        let txt2img = Txt2ImgRequest {
            negative_prompt: Some("test".to_string()),
            ..Txt2ImgRequest::default()
        };
        let img2img = Img2ImgRequest {
            negative_prompt: Some("test".to_string()),
            ..Img2ImgRequest::default()
        };
        assert!(matches!(
            map_settings()
                .endpoint(
                    |(txt2img, img2img): (Txt2ImgRequest, Img2ImgRequest)| async {
                        assert!(
                            (txt2img, img2img)
                                == (
                                    Txt2ImgRequest {
                                        negative_prompt: Some("test".to_string()),
                                        ..Txt2ImgRequest::default()
                                    },
                                    Img2ImgRequest {
                                        negative_prompt: Some("test".to_string()),
                                        ..Img2ImgRequest::default()
                                    }
                                )
                        );
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![
                    ConfigParameters::default(),
                    State::Ready {
                        bot_state: BotState::Generate,
                        txt2img,
                        img2img
                    }
                ])
                .await,
            ControlFlow::Break(_)
        ));
    }
}
