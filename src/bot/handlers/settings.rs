use anyhow::anyhow;
use teloxide::{
    dispatching::UpdateHandler,
    dptree::case,
    macros::BotCommands,
    payloads::setters::*,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::api::{Img2ImgRequest, Txt2ImgRequest};

use super::{DiffusionDialogue, State};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Authenticated commands")]
pub enum SettingsCommands {
    #[command(description = "txt2img settings")]
    Txt2ImgSettings,
    #[command(description = "img2img settings")]
    Img2ImgSettings,
}

#[allow(dead_code)]
pub struct Settings {
    pub steps: u32,
    pub seed: i64,
    pub batch_size: u32,
    pub n_iter: u32,
    pub cfg_scale: f64,
    pub width: u32,
    pub height: u32,
    pub negative_prompt: String,
    pub styles: Vec<String>,
    pub restore_faces: bool,
    pub tiling: bool,
    pub denoising_strength: Option<f64>,
    pub sampler_index: String,
}

impl Settings {
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
            steps: value.steps.ok_or(anyhow!("Missing steps!"))?,
            seed: value.seed.ok_or(anyhow!("Missing seed!"))?,
            batch_size: value.batch_size.ok_or(anyhow!("Missing batch_size!"))?,
            n_iter: value.n_iter.ok_or(anyhow!("Missing n_iter!"))?,
            cfg_scale: value.cfg_scale.ok_or(anyhow!("Missing cfg_scale!"))?,
            width: value.width.ok_or(anyhow!("Missing width!"))?,
            height: value.height.ok_or(anyhow!("Missing height!"))?,
            negative_prompt: value
                .negative_prompt
                .clone()
                .ok_or(anyhow!("Missing negative_prompt!"))?,
            styles: value.styles.clone().ok_or(anyhow!("Missing styles!"))?,
            restore_faces: value
                .restore_faces
                .ok_or(anyhow!("Missing restore_faces!"))?,
            tiling: value.tiling.ok_or(anyhow!("Missing tiling!"))?,
            denoising_strength: value.denoising_strength,
            sampler_index: value
                .sampler_index
                .clone()
                .ok_or(anyhow!("Missing sampler_index!"))?,
        })
    }
}

impl TryFrom<Txt2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: Txt2ImgRequest) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&Img2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: &Img2ImgRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            steps: value.steps.ok_or(anyhow!("Missing steps!"))?,
            seed: value.seed.ok_or(anyhow!("Missing seed!"))?,
            batch_size: value.batch_size.ok_or(anyhow!("Missing batch_size!"))?,
            n_iter: value.n_iter.ok_or(anyhow!("Missing n_iter!"))?,
            cfg_scale: value.cfg_scale.ok_or(anyhow!("Missing cfg_scale!"))?,
            width: value.width.ok_or(anyhow!("Missing width!"))?,
            height: value.height.ok_or(anyhow!("Missing height!"))?,
            negative_prompt: value
                .negative_prompt
                .clone()
                .ok_or(anyhow!("Missing negative_prompt!"))?,
            styles: value.styles.clone().ok_or(anyhow!("Missing styles!"))?,
            restore_faces: value
                .restore_faces
                .ok_or(anyhow!("Missing restore_faces!"))?,
            tiling: value.tiling.ok_or(anyhow!("Missing tiling!"))?,
            denoising_strength: value.denoising_strength,
            sampler_index: value
                .sampler_index
                .clone()
                .ok_or(anyhow!("Missing sampler_index!"))?,
        })
    }
}

impl TryFrom<Img2ImgRequest> for Settings {
    type Error = anyhow::Error;

    fn try_from(value: Img2ImgRequest) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

pub(crate) async fn handle_settings(
    bot: Bot,
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

    let settings = if parent.photo().is_some() {
        dialogue
            .update(State::SettingsImg2Img {
                selection: None,
                txt2img,
                img2img: img2img.clone(),
            })
            .await
            .map_err(|e| anyhow!(e))?;

        Settings::try_from(img2img)?
    } else if parent.text().is_some() {
        dialogue
            .update(State::SettingsTxt2Img {
                selection: None,
                txt2img: txt2img.clone(),
                img2img,
            })
            .await
            .map_err(|e| anyhow!(e))?;

        Settings::try_from(txt2img)?
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
            .update(State::Ready { txt2img, img2img })
            .await
            .map_err(|e| anyhow!(e))?;
        bot.answer_callback_query(q.id).await?;
        bot.edit_message_text(message.chat.id, message.id, "Please enter a prompt.")
            .reply_markup(InlineKeyboardMarkup::new([[]]))
            .await?;
        return Ok(());
    }

    let mut state = dialogue.get_or_default().await.map_err(|e| anyhow!(e))?;
    match &mut state {
        State::SettingsTxt2Img { selection, .. } | State::SettingsImg2Img { selection, .. } => {
            *selection = Some(setting.to_string())
        }
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

pub(crate) async fn handle_settings_value(
    bot: Bot,
    dialogue: DiffusionDialogue,
    msg: Message,
    text: String,
) -> anyhow::Result<()> {
    let mut state = dialogue.get_or_default().await.map_err(|e| anyhow!(e))?;

    let settings = match &mut state {
        State::SettingsTxt2Img {
            selection, txt2img, ..
        } => {
            if let Some(setting) = selection {
                if let Err(e) = update_txt2img_setting(txt2img, setting, text) {
                    bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                        .await?;
                    return Ok(());
                }
            }
            *selection = None;
            Settings::try_from(&*txt2img)?
        }
        State::SettingsImg2Img {
            selection, img2img, ..
        } => {
            if let Some(setting) = selection {
                if let Err(e) = update_img2img_setting(img2img, setting, text) {
                    bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                        .await?;
                    return Ok(());
                }
            }
            *selection = None;
            Settings::try_from(&*img2img)?
        }
        _ => return Err(anyhow!("Invalid settings state")),
    };

    dialogue.update(state).await.map_err(|e| anyhow!(e))?;

    bot.send_message(msg.chat.id, "Please make a selection.")
        .reply_markup(settings.keyboard())
        .await?;

    Ok(())
}

pub(crate) fn settings_schema() -> UpdateHandler<anyhow::Error> {
    let settings_command_handler =
        Update::filter_message()
            .filter_command::<SettingsCommands>()
            .endpoint(
                |msg: Message,
                 bot: Bot,
                 dialogue: DiffusionDialogue,
                 cmd: SettingsCommands| async move {
                    let (txt2img, img2img) =
                        match dialogue.get_or_default().await.map_err(|e| anyhow!(e))? {
                            State::Ready { txt2img, img2img }
                            | State::SettingsTxt2Img {
                                txt2img, img2img, ..
                            }
                            | State::SettingsImg2Img {
                                txt2img, img2img, ..
                            } => (txt2img, img2img),
                        };
                    match cmd {
                        SettingsCommands::Img2ImgSettings => {
                            dialogue
                                .update(State::SettingsImg2Img {
                                    selection: None,
                                    txt2img,
                                    img2img: img2img.clone(),
                                })
                                .await
                                .map_err(|e| anyhow!(e))?;
                            let settings = Settings::try_from(img2img)?;
                            bot.send_message(msg.chat.id, "Please make a selection.")
                                .reply_markup(settings.keyboard())
                                .send()
                                .await?;
                            Ok(())
                        }
                        SettingsCommands::Txt2ImgSettings => {
                            dialogue
                                .update(State::SettingsTxt2Img {
                                    selection: None,
                                    txt2img: txt2img.clone(),
                                    img2img,
                                })
                                .await
                                .map_err(|e| anyhow!(e))?;
                            let settings = Settings::try_from(txt2img)?;
                            bot.send_message(msg.chat.id, "Please make a selection.")
                                .reply_markup(settings.keyboard())
                                .send()
                                .await?;
                            Ok(())
                        }
                    }
                },
            );
    let callback_handler = Update::filter_callback_query()
        .chain(dptree::filter(|q: CallbackQuery| {
            if let Some(data) = q.data {
                data.starts_with("settings")
            } else {
                false
            }
        }))
        .branch(case![State::Ready { txt2img, img2img }].endpoint(handle_settings))
        .branch(
            case![State::SettingsImg2Img {
                selection,
                txt2img,
                img2img
            }]
            .endpoint(handle_settings_button),
        )
        .branch(
            case![State::SettingsTxt2Img {
                selection,
                txt2img,
                img2img
            }]
            .endpoint(handle_settings_button),
        );

    let message_handler = Update::filter_message()
        .branch(
            Message::filter_text()
                .branch(
                    case![State::SettingsTxt2Img {
                        selection,
                        txt2img,
                        img2img
                    }]
                    .endpoint(handle_settings_value),
                )
                .branch(
                    case![State::SettingsImg2Img {
                        selection,
                        txt2img,
                        img2img
                    }]
                    .endpoint(handle_settings_value),
                ),
        )
        .branch(
            case![State::SettingsTxt2Img {
                selection,
                txt2img,
                img2img
            }]
            .endpoint(|bot: Bot, msg: Message| async move {
                bot.send_message(msg.chat.id, "Please enter a valid value.")
                    .await?;
                Ok(())
            }),
        )
        .branch(
            case![State::SettingsImg2Img {
                selection,
                txt2img,
                img2img
            }]
            .endpoint(|bot: Bot, msg: Message| async move {
                bot.send_message(msg.chat.id, "Please enter a valid value.")
                    .await?;
                Ok(())
            }),
        );
    dptree::entry()
        .branch(settings_command_handler)
        .branch(message_handler)
        .branch(callback_handler)
}
