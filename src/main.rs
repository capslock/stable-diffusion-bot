use anyhow::Context;
use teloxide::prelude::*;
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder().pretty().with_target(true).finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("setting default subscriber failed")?;

    LogTracer::init()?;

    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;

    Ok(())
}
