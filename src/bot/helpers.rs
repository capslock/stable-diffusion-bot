use anyhow::Context;
use futures::TryStreamExt;
use teloxide::{net::Download, types::File, Bot};

pub(crate) async fn get_file(bot: &Bot, file: &File) -> anyhow::Result<bytes::Bytes> {
    bot.download_file_stream(&file.path)
        .try_collect()
        .await
        .context("Failed to download file")
        .map(bytes::BytesMut::freeze)
}
