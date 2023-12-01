use anyhow::Context;
use reqwest::Url;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{Prompt, Response};

#[derive(Serialize, Debug)]
#[skip_serializing_none]
struct PromptWrapper<'a> {
    prompt: &'a Prompt,
    client_id: Option<uuid::Uuid>,
}

/// Struct representing a connection to the ComfyUI API `prompt` endpoint.
pub struct PromptApi {
    client: reqwest::Client,
    endpoint: Url,
    client_id: uuid::Uuid,
}

impl PromptApi {
    /// Constructs a new `PromptApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `PromptApi` instance on success, or an error if url parsing failed.
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
    pub async fn send(&self, prompt: &Prompt) -> anyhow::Result<Response> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&PromptWrapper {
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
