use std::collections::HashSet;

use anyhow::Context;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{
        self, serializer::Json, ErasedStorage, InMemStorage, SqliteStorage, Storage,
    },
    dptree::case,
    prelude::*,
    types::{MediaPhoto, Update, UserId},
    utils::command::BotCommands,
};
use tracing::{metadata::LevelFilter, warn};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, FmtSubscriber};

#[derive(Serialize, Deserialize, Default, Debug)]
struct Config {
    api_key: String,
    allowed_users: Vec<u64>,
    db_path: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum State {
    #[default]
    Start,
    Next,
}

type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

type DiffusionDialogue = Dialogue<State, ErasedStorage<State>>;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder().pretty().with_target(true).finish();

    tracing::subscriber::set_global_default(
        subscriber.with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .from_env_lossy(),
        ),
    )
    .context("setting default subscriber failed")?;

    LogTracer::init()?;

    let config: Config = Figment::new()
        .merge(("config.allowed_users", Vec::<u64>::new()))
        .merge(Toml::file("config.toml"))
        .merge(Env::prefixed("SD_TELEGRAM_"))
        .extract()
        .context("Invalid configuration")?;

    let storage: DialogueStorage = if let Some(path) = config.db_path {
        SqliteStorage::open(&path, Json).await.unwrap().erase()
    } else {
        InMemStorage::new().erase()
    };

    let bot = Bot::new(config.api_key);

    let allowed_users = config.allowed_users.into_iter().map(UserId).collect();

    let parameters = ConfigParameters { allowed_users };

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
        .branch(Message::filter_photo().endpoint(
            |bot: Bot, msg: Message, photo: MediaPhoto| async move {
                bot.send_message(msg.chat.id, format!("got image {}", photo.photo[0].file.id))
                    .reply_to_message_id(msg.id)
                    .await?;
                Ok(())
            },
        ))
        .branch(case![State::Start].endpoint(
            |bot: Bot, dialogue: DiffusionDialogue, msg: Message| async move {
                dialogue
                    .update(State::Next)
                    .await
                    .expect("Failed to update state");
                bot.send_message(
                    msg.chat.id,
                    format!("{:?}", dialogue.get_or_default().await.unwrap_or_default()),
                )
                .reply_to_message_id(msg.id)
                .await?;
                Ok(())
            },
        ))
        .branch(case![State::Next].endpoint(
            |bot: Bot, dialogue: DiffusionDialogue, msg: Message| async move {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "next: {:?}",
                        dialogue.get_or_default().await.unwrap_or_default()
                    ),
                )
                .await?;
                Ok(())
            },
        ));

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

#[derive(Clone, Debug)]
struct ConfigParameters {
    allowed_users: HashSet<UserId>,
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
) -> Result<(), teloxide::RequestError> {
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
