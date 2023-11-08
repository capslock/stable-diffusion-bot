use anyhow::anyhow;
use teloxide::{dispatching::UpdateHandler, prelude::*, utils::command::BotCommands};

use super::{ConfigParameters, DiffusionDialogue, State};

mod image;
pub(crate) use image::*;

mod settings;
pub(crate) use settings::*;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Simple commands")]
pub(crate) enum UnauthenticatedCommands {
    #[command(description = "show help message.")]
    Help,
    #[command(description = "start the bot.")]
    Start,
    #[command(description = "change settings.")]
    Settings,
}

pub(crate) async fn unauthenticated_commands_handler(
    cfg: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: UnauthenticatedCommands,
    dialogue: DiffusionDialogue,
) -> anyhow::Result<()> {
    let text = match cmd {
        UnauthenticatedCommands::Help => {
            if cfg.chat_is_allowed(&msg.chat.id)
                || cfg.chat_is_allowed(&msg.from().unwrap().id.into())
            {
                format!(
                    "{}\n\n{}\n\n{}",
                    UnauthenticatedCommands::descriptions(),
                    SettingsCommands::descriptions(),
                    GenCommands::descriptions()
                )
            } else if msg.chat.is_group() || msg.chat.is_supergroup() {
                UnauthenticatedCommands::descriptions()
                    .username_from_me(&me)
                    .to_string()
            } else {
                UnauthenticatedCommands::descriptions().to_string()
            }
        }
        UnauthenticatedCommands::Start => {
            dialogue
                .update(State::Ready {
                    txt2img: cfg.txt2img_defaults,
                    img2img: cfg.img2img_defaults,
                })
                .await
                .map_err(|e| anyhow!(e))?;
            "This bot generates images using stable diffusion! Enter a prompt to get started!"
                .to_owned()
        }
        UnauthenticatedCommands::Settings => "Sorry, not yet implemented.".to_owned(),
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}

pub(crate) fn auth_filter() -> UpdateHandler<anyhow::Error> {
    dptree::filter(|cfg: ConfigParameters, upd: Update| {
        upd.chat()
            .map(|chat| cfg.chat_is_allowed(&chat.id))
            .unwrap_or_default()
            || upd
                .user()
                .map(|user| cfg.chat_is_allowed(&user.id.into()))
                .unwrap_or_default()
    })
}

pub(crate) fn unauth_command_handler() -> UpdateHandler<anyhow::Error> {
    Update::filter_message().chain(
        teloxide::filter_command::<UnauthenticatedCommands, _>()
            .endpoint(unauthenticated_commands_handler),
    )
}

pub(crate) fn authenticated_command_handler() -> UpdateHandler<anyhow::Error> {
    auth_filter()
        .branch(settings_schema())
        .branch(image_schema())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};
    use teloxide::types::{Me, UpdateKind, User};

    fn create_message() -> Message {
        let json = r#"{
          "message_id": 123456,
          "from": {
           "id": 123456789,
           "is_bot": false,
           "first_name": "Stable",
           "last_name": "Diffusion",
           "username": "sd",
           "language_code": "en"
          },
          "chat": {
           "id": 123456789,
           "first_name": "Stable",
           "last_name": "Diffusion",
           "username": "sd",
           "type": "private"
          },
          "date": 1234567890,
          "text": "hi"
         }"#;
        serde_json::from_str::<Message>(json).unwrap()
    }
    fn create_me() -> Me {
        Me {
            user: User {
                id: UserId(123456780),
                is_bot: true,
                first_name: "Stable Diffusion".to_string(),
                last_name: None,
                username: Some("sdbot".to_string()),
                language_code: Some("en".to_string()),
                is_premium: false,
                added_to_attachment_menu: false,
            },
            can_join_groups: false,
            can_read_all_group_messages: false,
            supports_inline_queries: false,
        }
    }

    fn create_config() -> ConfigParameters {
        ConfigParameters {
            allowed_users: HashSet::new(),
            api: stable_diffusion_api::Api::new(),
            txt2img_defaults: Txt2ImgRequest::default(),
            img2img_defaults: Img2ImgRequest::default(),
            allow_all_users: false,
        }
    }

    #[tokio::test]
    async fn test_unauth_command_handler() {
        let me = create_me();

        let msg = create_message();

        let update = Update {
            id: 1,
            kind: UpdateKind::Message(msg.clone()),
        };

        assert!(matches!(
            unauth_command_handler()
                .dispatch(dptree::deps![msg, update, me])
                .await,
            ControlFlow::Continue(_)
        ));
    }

    #[tokio::test]
    async fn test_auth_command_handler() {
        let cfg = create_config();

        let me = create_me();

        let msg = create_message();

        let update = Update {
            id: 1,
            kind: UpdateKind::Message(msg.clone()),
        };

        assert!(matches!(
            authenticated_command_handler()
                .dispatch(dptree::deps![msg, update, me, cfg])
                .await,
            ControlFlow::Continue(_)
        ));
    }
}
