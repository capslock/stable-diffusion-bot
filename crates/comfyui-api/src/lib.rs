use anyhow::{anyhow, Context};
use futures_util::{Stream, StreamExt};
use reqwest::Url;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::warn;

mod prompt;
pub use prompt::*;

mod history;
pub use history::*;

mod websocket;
pub use websocket::*;

/// Struct representing a connection to a Stable Diffusion WebUI API.
#[derive(Clone, Debug)]
pub struct Api {
    client: reqwest::Client,
    url: Url,
    client_id: uuid::Uuid,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            url: Url::parse("http://localhost:8188").expect("Failed to parse default URL"),
            client_id: uuid::Uuid::new_v4(),
        }
    }
}

impl Api {
    /// Returns a new `Api` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `Api` instance with the given URL as a string value.
    ///
    /// # Arguments
    ///
    /// * `url` - A string that specifies the Stable Diffusion WebUI API URL endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn new_with_url<S>(url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            url: Url::parse(url.as_ref()).context("Failed to parse URL")?,
            ..Default::default()
        })
    }

    /// Returns a new `Api` instance with the given `reqwest::Client` and URL as a string value.
    ///
    /// # Arguments
    ///
    /// * `client` - An instance of `reqwest::Client`.
    /// * `url` - A string that specifies the Stable Diffusion WebUI API URL endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn new_with_client_and_url<S>(client: reqwest::Client, url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            client,
            url: Url::parse(url.as_ref()).context("Failed to parse URL")?,
            ..Default::default()
        })
    }

    pub fn prompt(&self) -> anyhow::Result<Comfy> {
        Ok(Comfy::new_with_url(
            self.client.clone(),
            self.url
                .join("prompt")
                .context("Failed to parse comfyUI endpoint")?,
            self.client_id,
        ))
    }

    pub fn history(&self) -> anyhow::Result<History> {
        Ok(History::new_with_url(
            self.client.clone(),
            self.url
                .join("history/")
                .context("Failed to parse history endpoint")?,
        ))
    }

    pub fn websocket(&self) -> anyhow::Result<Websocket> {
        let mut url = self.url.clone();
        url.set_scheme("ws")
            .map_err(|_| anyhow!("Failed to set scheme: ws://"))?;
        Ok(Websocket::new_with_url(
            url.join(format!("ws?clientId={}", self.client_id).as_str())
                .context("Failed to parse websocket endpoint")?,
        ))
    }
}

mod request {
    use serde::{Deserialize, Serialize};
    use serde_with::skip_serializing_none;

    use crate::prompt;

    #[derive(Default, Serialize, Deserialize, Debug)]
    #[skip_serializing_none]
    pub(crate) struct Prompt {
        pub prompt: prompt::Prompt,
        pub client_id: Option<uuid::Uuid>,
    }
}

pub struct Comfy {
    client: reqwest::Client,
    endpoint: Url,
    client_id: uuid::Uuid,
}

impl Comfy {
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
    pub fn new(
        client: reqwest::Client,
        endpoint: String,
        client_id: uuid::Uuid,
    ) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
            client_id,
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
    pub fn new_with_url(client: reqwest::Client, endpoint: Url, client_id: uuid::Uuid) -> Self {
        Self {
            client,
            endpoint,
            client_id,
        }
    }

    /// Sends an image request using the Txt2Img client.
    ///
    /// # Arguments
    ///
    /// * `request` - An Txt2ImgRequest containing the parameters for the image request.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ImgResponse<Txt2ImgRequest>` on success, or an error if one occurred.
    pub async fn send(&self, prompt: Prompt) -> anyhow::Result<Response> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&request::Prompt {
                prompt,
                client_id: Some(self.client_id),
            })
            .send()
            .await
            .context("failed to send request")?;
        if response.status().is_success() {
            return response.json().await.context("failed to parse json");
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .context("failed to get response text")?;
        Err(anyhow::anyhow!(
            "got error code: {}, message text: {}",
            status,
            text
        ))
    }
}

pub struct Websocket {
    endpoint: Url,
}

impl Websocket {
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

pub struct History {
    client: reqwest::Client,
    endpoint: Url,
}

impl History {
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
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
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
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    pub async fn get(&self, prompt_id: uuid::Uuid) -> anyhow::Result<HistoryOrUnknown> {
        let response = self
            .client
            .get(
                self.endpoint
                    .clone()
                    .join(prompt_id.to_string().as_str())
                    .context("failed to parse url")?,
            )
            .send()
            .await
            .context("failed to send request")?;
        if response.status().is_success() {
            return response.json().await.context("failed to parse json");
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .context("failed to get response text")?;
        Err(anyhow::anyhow!(
            "got error code: {}, message text: {}",
            status,
            text
        ))
    }
}
