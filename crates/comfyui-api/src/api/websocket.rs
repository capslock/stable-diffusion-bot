use anyhow::{anyhow, Context};
use futures_util::{Stream, StreamExt};
use reqwest::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::warn;

use crate::{Preview, PreviewOrUpdate, UpdateOrUnknown};

/// Struct representing a connection to the ComfyUI API `ws` endpoint.
pub struct WebsocketApi {
    endpoint: Url,
}

impl WebsocketApi {
    /// Constructs a new `WebsocketApi` client with a given ComfyUI API endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `WebsocketApi` instance on success, or an error if url parsing failed.
    pub fn new(endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
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

    /// Connects to the websocket endpoint and returns a stream of `PreviewOrUpdate` values.
    ///
    /// # Returns
    ///
    /// A `Stream` of `PreviewOrUpdate` values. These are either `Update` values, which contain
    /// progress updates for a task, or `Preview` values, which contain a preview image.
    pub async fn connect(
        &self,
    ) -> anyhow::Result<impl Stream<Item = Result<PreviewOrUpdate, anyhow::Error>> + Unpin> {
        let (connection, _) = connect_async(&self.endpoint)
            .await
            .context("WebSocket connection failed")?;
        Ok(Box::pin(connection.filter_map(|m| async {
            match m {
                Ok(m) => match m {
                    Message::Text(t) => Some(
                        serde_json::from_str::<UpdateOrUnknown>(t.as_str())
                            .context("failed to parse websocket message text")
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
        })))
    }
}
