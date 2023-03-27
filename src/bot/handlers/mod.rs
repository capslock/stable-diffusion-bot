use teloxide::{prelude::*, utils::command::BotCommands};

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
) -> anyhow::Result<()> {
    let text = match cmd {
        UnauthenticatedCommands::Help => {
            if cfg.allowed_users.contains(&msg.from().unwrap().id) {
                format!(
                    "{}\n\n{}",
                    UnauthenticatedCommands::descriptions(),
                    SettingsCommands::descriptions()
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
            "This bot generates images using stable diffusion! Enter a prompt to get started!"
                .to_owned()
        }
        UnauthenticatedCommands::Settings => "Sorry, not yet implemented.".to_owned(),
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
