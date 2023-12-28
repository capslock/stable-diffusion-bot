use anyhow::anyhow;
use itertools::Itertools as _;
use sal_e_api::GenParams;
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
    pub steps: Option<u32>,
    // Random seed.
    pub seed: Option<i64>,
    // Number of images to generate per batch.
    pub batch_size: Option<u32>,
    // Number of batches of images to generate.
    pub n_iter: Option<u32>,
    // CFG scale.
    pub cfg_scale: Option<f32>,
    // Image width.
    pub width: Option<u32>,
    // Image height.
    pub height: Option<u32>,
    // Negative prompt.
    pub negative_prompt: Option<String>,
    // Denoising strength. Only used for img2img.
    pub denoising_strength: Option<f32>,
    // Sampler name.
    pub sampler_index: Option<String>,
}

impl Settings {
    /// Build an inline keyboard to configure settings.
    pub fn keyboard(&self) -> InlineKeyboardMarkup {
        InlineKeyboardMarkup::new(
            [
                self.steps.map(|steps| {
                    InlineKeyboardButton::callback(format!("Steps: {}", steps), "settings_steps")
                }),
                self.seed.map(|seed| {
                    InlineKeyboardButton::callback(format!("Seed: {}", seed), "settings_seed")
                }),
                self.n_iter.map(|n_iter| {
                    InlineKeyboardButton::callback(
                        format!("Batch Count: {}", n_iter),
                        "settings_count",
                    )
                }),
                self.cfg_scale.map(|cfg_scale| {
                    InlineKeyboardButton::callback(
                        format!("CFG Scale: {}", cfg_scale),
                        "settings_cfg",
                    )
                }),
                self.width.map(|width| {
                    InlineKeyboardButton::callback(format!("Width: {}", width), "settings_width")
                }),
                self.height.map(|height| {
                    InlineKeyboardButton::callback(format!("Height: {}", height), "settings_height")
                }),
                self.negative_prompt.as_ref().map(|_| {
                    InlineKeyboardButton::callback(
                        "Negative Prompt".to_owned(),
                        "settings_negative",
                    )
                }),
                self.denoising_strength.map(|denoising_strength| {
                    InlineKeyboardButton::callback(
                        format!("Denoising Strength: {}", denoising_strength),
                        "settings_denoising",
                    )
                }),
                Some(InlineKeyboardButton::callback(
                    "Cancel".to_owned(),
                    "settings_back",
                )),
            ]
            .into_iter()
            .flatten()
            .chunks(2)
            .into_iter()
            .map(Iterator::collect)
            .collect::<Vec<Vec<_>>>(),
        )
    }
}

impl From<&dyn GenParams> for Settings {
    fn from(value: &dyn GenParams) -> Self {
        Self {
            steps: value.steps(),
            seed: value.seed(),
            batch_size: value.batch_size(),
            n_iter: value.count(),
            cfg_scale: value.cfg(),
            width: value.width(),
            height: value.height(),
            negative_prompt: value.negative_prompt().clone(),
            denoising_strength: value.denoising(),
            sampler_index: value.sampler().clone(),
        }
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
    (txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
    q: CallbackQuery,
    chat_id: ChatId,
    parent: Message,
) -> anyhow::Result<()> {
    let settings = if parent.photo().is_some() {
        let settings = Settings::from(img2img.as_ref());
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
        let settings = Settings::from(txt2img.as_ref());
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

    bot.answer_callback_query(q.id).await?;
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
    (_, txt2img, img2img): (Option<String>, Box<dyn GenParams>, Box<dyn GenParams>),
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
        if let Err(e) = bot.delete_message(message.chat.id, message.id).await {
            error!("Failed to delete message: {:?}", e);
            bot.edit_message_text(message.chat.id, message.id, "Please enter a prompt.")
                .reply_markup(InlineKeyboardMarkup::new([[]]))
                .await?;
        }
        return Ok(());
    }

    let mut state = dialogue
        .get()
        .await
        .map_err(|e| anyhow!(e))?
        .unwrap_or_else(|| {
            State::new_with_defaults(
                cfg.txt2img_api.gen_params(None),
                cfg.img2img_api.gen_params(None),
            )
        });
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
    txt2img: &mut dyn GenParams,
    setting: S1,
    value: S2,
) -> anyhow::Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let value = value.as_ref();
    match setting.as_ref() {
        "steps" => txt2img.set_steps(value.parse()?),
        "seed" => txt2img.set_seed(value.parse()?),
        "count" => txt2img.set_count(value.parse()?),
        "cfg" => txt2img.set_cfg(value.parse()?),
        "width" => txt2img.set_width(value.parse()?),
        "height" => txt2img.set_height(value.parse()?),
        "negative" => txt2img.set_negative_prompt(value.to_owned()),
        "denoising" => txt2img.set_denoising(value.parse()?),
        _ => return Err(anyhow!("Got invalid setting: {}", setting.as_ref())),
    }
    Ok(())
}

fn update_img2img_setting<S1, S2>(
    img2img: &mut dyn GenParams,
    setting: S1,
    value: S2,
) -> anyhow::Result<()>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let value = value.as_ref();
    match setting.as_ref() {
        "steps" => img2img.set_steps(200.min(value.parse()?)),
        "seed" => img2img.set_seed((-1).max(value.parse()?)),
        "count" => img2img.set_count(value.parse::<u32>()?.clamp(1, 10)),
        "cfg" => img2img.set_cfg(value.parse::<f32>()?.clamp(0.0, 20.0)),
        "width" => img2img.set_width({
            let mut value = value.parse::<u32>()?;
            value -= value % 64;
            value.clamp(64, 1024)
        }),
        "height" => img2img.set_height({
            let mut value = value.parse::<u32>()?;
            value -= value % 64;
            value.clamp(64, 1024)
        }),
        "negative" => img2img.set_negative_prompt(value.to_owned()),
        "denoising" => img2img.set_denoising(value.parse::<f32>()?.clamp(0.0, 1.0)),
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
                State::new_with_defaults(
                    cfg.txt2img_api.gen_params(None),
                    cfg.img2img_api.gen_params(None),
                )
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
    (selection, mut txt2img, img2img): (Option<String>, Box<dyn GenParams>, Box<dyn GenParams>),
) -> anyhow::Result<()> {
    if let Some(ref setting) = selection {
        if let Err(e) = update_txt2img_setting(txt2img.as_mut(), setting, text) {
            bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                .await?;
            return Ok(());
        }
    }

    let bot_state = BotState::SettingsTxt2Img { selection: None };

    update_settings_value(
        bot,
        dialogue,
        msg.chat.id,
        Settings::from(txt2img.as_ref()),
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
    (selection, txt2img, mut img2img): (Option<String>, Box<dyn GenParams>, Box<dyn GenParams>),
) -> anyhow::Result<()> {
    if let Some(ref setting) = selection {
        if let Err(e) = update_img2img_setting(img2img.as_mut(), setting, text) {
            bot.send_message(msg.chat.id, format!("Please enter a valid value: {e:?}."))
                .await?;
            return Ok(());
        }
    }

    let bot_state = BotState::SettingsImg2Img { selection: None };

    update_settings_value(
        bot,
        dialogue,
        msg.chat.id,
        Settings::from(img2img.as_ref()),
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
        State::New => (
            cfg.txt2img_api.gen_params(None),
            cfg.img2img_api.gen_params(None),
        ),
    })
}

async fn handle_img2img_settings_command(
    msg: Message,
    bot: Bot,
    dialogue: DiffusionDialogue,
    (txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
) -> anyhow::Result<()> {
    let settings = Settings::from(img2img.as_ref());
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
    (txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>),
) -> anyhow::Result<()> {
    let settings = Settings::from(txt2img.as_ref());
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
    use async_trait::async_trait;
    use sal_e_api::{
        Img2ImgApi, Img2ImgApiError, Img2ImgParams, Response, Txt2ImgApi, Txt2ImgApiError,
        Txt2ImgParams,
    };
    use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
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
                    txt2img: Box::<Txt2ImgParams>::default(),
                    img2img: Box::<Img2ImgParams>::default()
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
                    txt2img: Box::<Txt2ImgParams>::default(),
                    img2img: Box::<Img2ImgParams>::default()
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
                    |(_, _, _): (Option<String>, Box<dyn GenParams>, Box<dyn GenParams>)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsTxt2Img { selection: None },
                    txt2img: Box::<Txt2ImgParams>::default(),
                    img2img: Box::<Img2ImgParams>::default()
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
                    |(_, _, _): (Option<String>, Box<dyn GenParams>, Box<dyn GenParams>)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::Ready {
                    bot_state: BotState::SettingsImg2Img { selection: None },
                    txt2img: Box::<Txt2ImgParams>::default(),
                    img2img: Box::<Img2ImgParams>::default()
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
                    |(_, _, _): (Option<String>, &dyn GenParams, &dyn GenParams)| async {
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![State::New])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[derive(Debug, Clone, Default)]
    struct MockApi;

    #[async_trait]
    impl Txt2ImgApi for MockApi {
        fn gen_params(&self, _user_params: Option<&dyn GenParams>) -> Box<dyn GenParams> {
            Box::<Txt2ImgParams>::default()
        }

        async fn txt2img(&self, _config: &dyn GenParams) -> Result<Response, Txt2ImgApiError> {
            Err(anyhow!("Not implemented"))?
        }
    }

    #[async_trait]
    impl Img2ImgApi for MockApi {
        fn gen_params(&self, _user_params: Option<&dyn GenParams>) -> Box<dyn GenParams> {
            Box::<Img2ImgParams>::default()
        }

        async fn img2img(&self, _config: &dyn GenParams) -> Result<Response, Img2ImgApiError> {
            Err(anyhow!("Not implemented"))?
        }
    }

    #[tokio::test]
    async fn test_map_settings_default() {
        assert!(matches!(
            map_settings()
                .endpoint(
                    |(txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>)| async move {
                        let txt2img = txt2img
                            .as_ref()
                            .as_any()
                            .downcast_ref::<Txt2ImgParams>()
                            .unwrap();
                        let img2img = img2img
                            .as_ref()
                            .as_any()
                            .downcast_ref::<Img2ImgParams>()
                            .unwrap();
                        assert!(
                            (txt2img, img2img)
                                == (&Txt2ImgParams::default(), &Img2ImgParams::default())
                        );
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![
                    ConfigParameters {
                        txt2img_api: Box::new(MockApi),
                        img2img_api: Box::new(MockApi),
                        allowed_users: Default::default(),
                        allow_all_users: false
                    },
                    State::New
                ])
                .await,
            ControlFlow::Break(_)
        ));
    }

    #[tokio::test]
    async fn test_map_settings_ready() {
        let txt2img = Txt2ImgParams {
            user_params: Txt2ImgRequest {
                negative_prompt: Some("test".to_string()),
                ..Txt2ImgRequest::default()
            },
            defaults: Some(Txt2ImgRequest::default()),
        };
        let img2img = Img2ImgParams {
            user_params: Img2ImgRequest {
                negative_prompt: Some("test".to_string()),
                ..Img2ImgRequest::default()
            },
            defaults: Some(Img2ImgRequest::default()),
        };
        assert!(matches!(
            map_settings()
                .endpoint(
                    |(txt2img, img2img): (Box<dyn GenParams>, Box<dyn GenParams>)| async move {
                        let txt2img = txt2img
                            .as_ref()
                            .as_any()
                            .downcast_ref::<Txt2ImgParams>()
                            .unwrap();
                        let img2img = img2img
                            .as_ref()
                            .as_any()
                            .downcast_ref::<Img2ImgParams>()
                            .unwrap();
                        assert!(
                            (txt2img, img2img)
                                == (
                                    &Txt2ImgParams {
                                        user_params: Txt2ImgRequest {
                                            negative_prompt: Some("test".to_string()),
                                            ..Txt2ImgRequest::default()
                                        },
                                        defaults: Some(Txt2ImgRequest::default()),
                                    },
                                    &Img2ImgParams {
                                        user_params: Img2ImgRequest {
                                            negative_prompt: Some("test".to_string()),
                                            ..Img2ImgRequest::default()
                                        },
                                        defaults: Some(Img2ImgRequest::default()),
                                    }
                                )
                        );
                        anyhow::Ok(())
                    }
                )
                .dispatch(dptree::deps![
                    ConfigParameters {
                        txt2img_api: Box::new(MockApi),
                        img2img_api: Box::new(MockApi),
                        allowed_users: Default::default(),
                        allow_all_users: false
                    },
                    State::Ready {
                        bot_state: BotState::Generate,
                        txt2img: Box::new(txt2img),
                        img2img: Box::new(img2img)
                    }
                ])
                .await,
            ControlFlow::Break(_)
        ));
    }
}
