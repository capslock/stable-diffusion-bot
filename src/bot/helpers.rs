use anyhow::Context;
use futures::TryStreamExt;
use teloxide::{net::Download, types::File, Bot};

/// Download a Telegram `File` and return its contents as bytes.
///
/// # Examples
///
/// ```no_run
/// use telegram_bot::*;
///
/// async fn handle_photo(bot: Bot, message: Message) -> Result<(), Box<dyn std::error::Error>> {
///     if let Some(photos) = message.photo(){
///         if let Some(photo) = photos.last() {
///             let file = bot.get_file(&photo.file.id).await?;
///             let bytes = get_file(&bot, &file)?;
///
///             // ... do something with the photo bytes ...
///
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub(crate) async fn get_file(bot: &Bot, file: &File) -> anyhow::Result<bytes::Bytes> {
    bot.download_file_stream(&file.path)
        .try_collect()
        .await
        .context("Failed to download file")
        .map(bytes::BytesMut::freeze)
}
