use reqwest::Url;

use crate::models::{History, Task};

/// Errors that can occur when interacting with `HistoryApi`.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum HistoryApiError {
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
    /// Server returned an error getting history
    #[error("Failed to get history: {status}: {error}")]
    GetHistoryFailed {
        status: reqwest::StatusCode,
        error: String,
    },
    /// Task was not found
    #[error("Task not found: {0}")]
    TaskNotFound(uuid::Uuid),
    /// Server returned an error getting task
    #[error("Failed to get task {task}: {status}: {error}")]
    GetTaskFailed {
        task: uuid::Uuid,
        status: reqwest::StatusCode,
        error: String,
    },
}

type Result<T> = std::result::Result<T, HistoryApiError>;

/// Struct representing a connection to the ComfyUI API `history` endpoint.
#[derive(Clone, Debug)]
pub struct HistoryApi {
    client: reqwest::Client,
    endpoint: Url,
}

impl HistoryApi {
    /// Constructs a new `HistoryApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `str` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `HistoryApi` instance on success, or an error if url parsing failed.
    pub fn new<S>(client: reqwest::Client, endpoint: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self::new_with_url(client, Url::parse(endpoint.as_ref())?))
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
    pub async fn get(&self) -> Result<History> {
        let response = self.client.get(self.endpoint.clone()).send().await?;
        if response.status().is_success() {
            return response
                .json()
                .await
                .map_err(HistoryApiError::InvalidResponse);
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(HistoryApiError::GetDataFailed)?;
        Err(HistoryApiError::GetHistoryFailed {
            status,
            error: text,
        })
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
    pub async fn get_prompt(&self, prompt_id: &uuid::Uuid) -> Result<Task> {
        let response = self
            .client
            .get(self.endpoint.clone().join(prompt_id.to_string().as_str())?)
            .send()
            .await?;
        if response.status().is_success() {
            let mut history: History = response
                .json()
                .await
                .map_err(HistoryApiError::InvalidResponse)?;
            return history
                .tasks
                .remove(prompt_id)
                .ok_or_else(|| HistoryApiError::TaskNotFound(*prompt_id));
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(HistoryApiError::GetDataFailed)?;
        Err(HistoryApiError::GetTaskFailed {
            task: *prompt_id,
            status,
            error: text,
        })
    }
}
