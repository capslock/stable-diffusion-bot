use reqwest::Url;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::models::{Prompt, Response};

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum PromptApiError {
    /// Error parsing endpoint URL
    #[error("Failed to parse endpoint URL")]
    ParseError(#[from] url::ParseError),
    /// Error sending request
    #[error("Failed to send request")]
    RequestFailed(#[from] reqwest::Error),
    /// An error occurred while parsing the response from the API.
    #[error("Parsing response failed")]
    InvalidResponse(#[source] reqwest::Error),
    /// An error occurred getting response data.
    #[error("Failed to get response data")]
    GetDataFailed(#[source] reqwest::Error),
    /// Server returned an error when sending prompt
    #[error("Failed to send prompt: {status}: {error}")]
    SendPromptFailed {
        status: reqwest::StatusCode,
        error: String,
    },
}

type Result<T> = std::result::Result<T, PromptApiError>;

#[derive(Serialize, Debug)]
#[skip_serializing_none]
struct PromptWrapper<'a> {
    prompt: &'a Prompt,
    client_id: Option<uuid::Uuid>,
}

/// Struct representing a connection to the ComfyUI API `prompt` endpoint.
#[derive(Clone, Debug)]
pub struct PromptApi {
    client: reqwest::Client,
    endpoint: Url,
    client_id: uuid::Uuid,
}

impl PromptApi {
    /// Constructs a new `PromptApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `str` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `PromptApi` instance on success, or an error if url parsing failed.
    pub fn new<S>(client: reqwest::Client, endpoint: S, client_id: uuid::Uuid) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self::new_with_url(
            client,
            Url::parse(endpoint.as_ref())?,
            client_id,
        ))
    }

    /// Constructs a new `PromptApi` client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new `PromptApi` instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url, client_id: uuid::Uuid) -> Self {
        Self {
            client,
            endpoint,
            client_id,
        }
    }

    /// Sends a prompt request using the `PromptApi` client.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A `Prompt` to send to the ComfyUI API.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Response` on success, or an error if the request failed.
    pub async fn send(&self, prompt: &Prompt) -> Result<Response> {
        self.send_as_client(prompt, self.client_id).await
    }

    async fn send_as_client(&self, prompt: &Prompt, client_id: uuid::Uuid) -> Result<Response> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&PromptWrapper {
                prompt,
                client_id: Some(client_id),
            })
            .send()
            .await?;
        if response.status().is_success() {
            return response
                .json()
                .await
                .map_err(PromptApiError::InvalidResponse);
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(PromptApiError::GetDataFailed)?;
        Err(PromptApiError::SendPromptFailed {
            status,
            error: text,
        })
    }

    /// Returns the client id used for requests.
    pub fn client_id(&self) -> uuid::Uuid {
        self.client_id
    }
}
