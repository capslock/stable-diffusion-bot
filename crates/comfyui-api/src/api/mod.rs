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

/// Errors that can occur opening API endpoints.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ApiError {
    /// Error parsing endpoint URL
    #[error("Failed to parse endpoint URL")]
    ParseError(#[from] url::ParseError),
    /// Error creating Prompt API
    #[error("Failed create prompt API")]
    CreatePromptApiFailed(#[from] PromptApiError),
    /// Error creating History API
    #[error("Failed create history API")]
    CreateHistoryApiFailed(#[from] HistoryApiError),
    /// Error creating Upload API
    #[error("Failed create upload API")]
    CreateUploadApiFailed(#[from] UploadApiError),
    /// Error creating View API
    #[error("Failed create view API")]
    CreateViewApiFailed(#[from] ViewApiError),
    /// Error parsing WebSocket endpoint API
    #[error("Failed parse websocket endpoint URL")]
    ParseWebSocketEndpointError(#[source] url::ParseError),
    /// Error setting WebSocket scheme
    #[error("Failed to set scheme: ws://{url}")]
    SetWebSocketSchemeFailed { url: url::Url },
}

type Result<T> = std::result::Result<T, ApiError>;

/// Struct representing a connection to a ComfyUI API.
#[derive(Clone, Debug)]
pub struct Api {
    client: reqwest::Client,
    url: Url,
    client_id: uuid::Uuid,
}

impl Default for Api {
    fn default() -> Self {
        Self::new().expect("Failed to parse default URL")
    }
}

impl Api {
    /// Returns a new `Api` instance with default settings.
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: Url::parse("http://localhost:8188")?,
            client_id: uuid::Uuid::new_v4(),
        })
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
    pub fn new_with_url<S>(url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            url: Url::parse(url.as_ref())?,
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
    pub fn new_with_client_and_url<S>(client: reqwest::Client, url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            client,
            url: Url::parse(url.as_ref())?,
            ..Default::default()
        })
    }

    /// Returns a new instance of `PromptApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `prompt` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn prompt(&self) -> Result<PromptApi> {
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
    pub fn prompt_with_client(&self, client_id: uuid::Uuid) -> Result<PromptApi> {
        Ok(PromptApi::new_with_url(
            self.client.clone(),
            self.url.join("prompt")?,
            client_id,
        ))
    }

    /// Returns a new instance of `HistoryApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `history` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn history(&self) -> Result<HistoryApi> {
        Ok(HistoryApi::new_with_url(
            self.client.clone(),
            self.url.join("history/")?,
        ))
    }

    /// Returns a new instance of `UploadApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `view` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn upload(&self) -> Result<UploadApi> {
        Ok(UploadApi::new_with_url(
            self.client.clone(),
            self.url.join("upload/")?,
        ))
    }

    /// Returns a new instance of `ViewApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `view` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn view(&self) -> Result<ViewApi> {
        Ok(ViewApi::new_with_url(
            self.client.clone(),
            self.url.join("view")?,
        ))
    }

    /// Returns a new instance of `WebsocketApi` with the API's cloned
    /// `reqwest::Client` and the URL for the `ws` endpoint.
    ///
    /// # Errors
    ///
    /// * If the URL fails to parse, an error will be returned.
    /// * On failure to set the `ws://` scheme on the URL, an error will be returned.
    pub fn websocket(&self) -> Result<WebsocketApi> {
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
    pub fn websocket_with_client(&self, client_id: uuid::Uuid) -> Result<WebsocketApi> {
        let mut url = self
            .url
            .clone()
            .join("ws")
            .map_err(ApiError::ParseWebSocketEndpointError)?;
        url.set_scheme("ws")
            .map_err(|_| ApiError::SetWebSocketSchemeFailed { url: url.clone() })?;
        url.set_query(Some(format!("clientId={}", client_id).as_str()));
        Ok(WebsocketApi::new_with_url(url))
    }
}
