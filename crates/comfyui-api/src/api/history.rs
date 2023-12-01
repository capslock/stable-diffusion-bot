use anyhow::Context;
use reqwest::Url;

use crate::{History, Task};

/// Struct representing a connection to the ComfyUI API `history` endpoint.
pub struct HistoryApi {
    client: reqwest::Client,
    endpoint: Url,
}

impl HistoryApi {
    /// Constructs a new `HistoryApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `HistoryApi` instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new `HistoryApi` client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new `HistoryApi` instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Sends a history request using the HistoryApi client.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `History` on success, or an error if the request failed.
    pub async fn get(&self) -> anyhow::Result<History> {
        let response = self
            .client
            .get(self.endpoint.clone())
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

    /// Sends a history request using the HistoryApi client.
    ///
    /// # Arguments
    ///
    /// * `prompt_id` - A `uuid::Uuid` representing the prompt id.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Task` on success, or an error if the request failed.
    pub async fn get_prompt(&self, prompt_id: &uuid::Uuid) -> anyhow::Result<Task> {
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
            let mut history: History = response.json().await.context("failed to parse json")?;
            return history
                .tasks
                .remove(prompt_id)
                .context("failed to get task");
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
