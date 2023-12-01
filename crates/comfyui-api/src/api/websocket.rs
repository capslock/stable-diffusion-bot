use anyhow::{anyhow, Context};
use futures_util::{Stream, StreamExt};
use reqwest::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::warn;

use crate::{Preview, PreviewOrUpdate, UpdateOrUnknown};

pub struct WebsocketApi {
    endpoint: Url,
}

impl WebsocketApi {
    /// Constructs a new Txt2Img client with a given `reqwest::Client` and Stable Diffusion API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new Txt2Img instance on success, or an error if url parsing failed.
    pub fn new(endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new Txt2Img client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new Txt2Img instance.
    pub fn new_with_url(endpoint: Url) -> Self {
        Self { endpoint }
    }

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
