use anyhow::{anyhow, Context};
use futures_util::{stream::FusedStream, StreamExt};
use reqwest::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::warn;

use crate::models::{Preview, PreviewOrUpdate, Update};

/// Struct representing a connection to the ComfyUI API `ws` endpoint.
#[derive(Clone, Debug)]
pub struct WebsocketApi {
    endpoint: Url,
}

impl WebsocketApi {
    /// Constructs a new `WebsocketApi` client with a given ComfyUI API endpoint.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - A `str` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `WebsocketApi` instance on success, or an error if url parsing failed.
    pub fn new<S>(endpoint: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self::new_with_url(
            Url::parse(endpoint.as_ref()).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new `WebsocketApi` client with a given endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new `WebsocketApi` instance.
    pub fn new_with_url(endpoint: Url) -> Self {
        Self { endpoint }
    }

    async fn connect_to_endpoint(
        &self,
        endpoint: &Url,
    ) -> anyhow::Result<impl FusedStream<Item = Result<PreviewOrUpdate, anyhow::Error>>> {
        let (connection, _) = connect_async(endpoint)
            .await
            .context("WebSocket connection failed")?;
        Ok(connection.filter_map(|m| async {
            match m {
                Ok(m) => match m {
                    Message::Text(t) => Some(
                        serde_json::from_str::<Update>(t.as_str())
                            .context(format!("failed to parse websocket message text: {}", t))
                            .map(PreviewOrUpdate::Update),
                    ),
                    Message::Binary(_) => {
                        Some(Ok(PreviewOrUpdate::Preview(Preview(m.into_data()))))
                    }
                    _ => {
                        warn!("unexpected websocket message type");
                        None
                    }
                },
                Err(e) => Some(Err(anyhow!("websocket error: {}", e))),
            }
        }))
    }

    async fn connect_impl(
        &self,
    ) -> anyhow::Result<impl FusedStream<Item = Result<PreviewOrUpdate, anyhow::Error>>> {
        self.connect_to_endpoint(&self.endpoint).await
    }

    /// Connects to the websocket endpoint and returns a stream of `PreviewOrUpdate` values.
    ///
    /// # Returns
    ///
    /// A `Stream` of `PreviewOrUpdate` values. These are either `Update` values, which contain
    /// progress updates for a task, or `Preview` values, which contain a preview image.
    pub async fn connect(
        &self,
    ) -> anyhow::Result<impl FusedStream<Item = Result<PreviewOrUpdate, anyhow::Error>>> {
        self.connect_impl().await
    }

    /// Connects to the websocket endpoint and returns a stream of `Update` values.
    ///
    /// # Returns
    ///
    /// A `Stream` of `Update` values. These contain progress updates for a task.
    pub async fn updates(
        &self,
    ) -> anyhow::Result<impl FusedStream<Item = Result<Update, anyhow::Error>>> {
        Ok(self.connect_impl().await?.filter_map(|m| async {
            match m {
                Ok(PreviewOrUpdate::Update(u)) => Some(Ok(u)),
                Ok(PreviewOrUpdate::Preview(_)) => None,
                Err(e) => Some(Err(e)),
            }
        }))
    }

    /// Connects to the websocket endpoint and returns a stream of `Preview` values.
    ///
    /// # Returns
    ///
    /// A `Stream` of `Preview` values. These contain preview images.
    pub async fn previews(
        &self,
    ) -> anyhow::Result<impl FusedStream<Item = Result<Preview, anyhow::Error>>> {
        Ok(self.connect_impl().await?.filter_map(|m| async {
            match m {
                Ok(PreviewOrUpdate::Update(_)) => None,
                Ok(PreviewOrUpdate::Preview(p)) => Some(Ok(p)),
                Err(e) => Some(Err(e)),
            }
        }))
    }
}
