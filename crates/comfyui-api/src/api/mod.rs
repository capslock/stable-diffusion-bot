use anyhow::{anyhow, Context};
use reqwest::Url;

pub mod history;
pub mod prompt;
pub mod upload;
pub mod view;
pub mod websocket;

pub use history::*;
pub use prompt::*;
pub use upload::*;
pub use view::*;
pub use websocket::*;

/// Struct representing a connection to a ComfyUI API.
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
    /// * `url` - A string that specifies the ComfyUI API URL endpoint.
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
    /// * `url` - A string that specifies the ComfyUI API URL endpoint.
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

    /// Returns a new instance of `PromptApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `prompt` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn prompt(&self) -> anyhow::Result<PromptApi> {
        self.prompt_with_client(self.client_id)
    }

    /// Returns a new instance of `PromptApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `prompt` endpoint and the
    /// specified client id.
    ///
    /// # Arguments
    ///
    /// * `client_id` - A `uuid::Uuid` representing the client id to use for the request.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn prompt_with_client(&self, client_id: uuid::Uuid) -> anyhow::Result<PromptApi> {
        Ok(PromptApi::new_with_url(
            self.client.clone(),
            self.url
                .join("prompt")
                .context("Failed to parse comfyUI endpoint")?,
            client_id,
        ))
    }

    /// Returns a new instance of `HistoryApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `history` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn history(&self) -> anyhow::Result<HistoryApi> {
        Ok(HistoryApi::new_with_url(
            self.client.clone(),
            self.url
                .join("history/")
                .context("Failed to parse history endpoint")?,
        ))
    }

    /// Returns a new instance of `UploadApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `view` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn upload(&self) -> anyhow::Result<UploadApi> {
        Ok(UploadApi::new_with_url(
            self.client.clone(),
            self.url
                .join("upload/")
                .context("Failed to parse view endpoint")?,
        ))
    }

    /// Returns a new instance of `ViewApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `view` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn view(&self) -> anyhow::Result<ViewApi> {
        Ok(ViewApi::new_with_url(
            self.client.clone(),
            self.url
                .join("view")
                .context("Failed to parse view endpoint")?,
        ))
    }

    /// Returns a new instance of `WebsocketApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `ws` endpoint.
    ///
    /// # Errors
    ///
    /// * If the URL fails to parse, an error will be returned.
    /// * On failure to set the `ws://` scheme on the URL, an error will be returned.
    pub fn websocket(&self) -> anyhow::Result<WebsocketApi> {
        self.websocket_with_client(self.client_id)
    }

    /// Returns a new instance of `WebsocketApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `ws` endpoint and the specified
    /// client id.
    ///
    /// # Arguments
    ///
    /// * `client_id` - A `uuid::Uuid` representing the client id to use for the request.
    ///
    /// # Errors
    ///
    /// * If the URL fails to parse, an error will be returned.
    /// * On failure to set the `ws://` scheme on the URL, an error will be returned.
    pub fn websocket_with_client(&self, client_id: uuid::Uuid) -> anyhow::Result<WebsocketApi> {
        let mut url = self
            .url
            .clone()
            .join("ws")
            .context("Failed to set websocket endpoint")?;
        url.set_scheme("ws")
            .map_err(|_| anyhow!("Failed to set scheme: ws://"))?;
        url.set_query(Some(format!("clientId={}", client_id).as_str()));
        Ok(WebsocketApi::new_with_url(url))
    }
}
