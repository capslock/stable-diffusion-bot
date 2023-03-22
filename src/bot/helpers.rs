use anyhow::Context;
use teloxide::{types::File, Bot};

pub(crate) async fn get_file(
    client: &reqwest::Client,
    bot: &Bot,
    file: &File,
) -> anyhow::Result<bytes::Bytes> {
    client
        .get(format!(
            "https://api.telegram.org/file/bot{}/{}",
            bot.token(),
            file.path
        ))
        .send()
        .await
        .context("Failed to get file")?
        .bytes()
        .await
        .context("Failed to get bytes")
}
