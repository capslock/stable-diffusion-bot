use reqwest::Url;

use crate::models::Image;

/// Errors that can occur when interacting with `ViewApi`.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ViewApiError {
    /// Error parsing endpoint URL
    #[error("Failed to parse endpoint URL")]
    ParseError(#[from] url::ParseError),
    /// Error sending request
    #[error("Failed to send request")]
    RequestFailed(#[from] reqwest::Error),
    /// An error occurred getting response bytes.
    #[error("Failed to get response bytes")]
    GetBytesFailed(#[source] reqwest::Error),
    /// An error occurred getting response text.
    #[error("Failed to get response text")]
    GetTextFailed(#[source] reqwest::Error),
    /// Server returned an error when uploading file
    #[error("Failed to upload image: {status}: {error}")]
    ViewImageFailed {
        status: reqwest::StatusCode,
        error: String,
    },
}

type Result<T> = std::result::Result<T, ViewApiError>;

/// Struct representing a connection to the ComfyUI API `view` endpoint.
#[derive(Clone, Debug)]
pub struct ViewApi {
    client: reqwest::Client,
    endpoint: Url,
}

impl ViewApi {
    /// Constructs a new `ViewApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `str` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `ViewApi` instance on success, or an error if url parsing failed.
    pub fn new<S>(client: reqwest::Client, endpoint: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self::new_with_url(client, Url::parse(endpoint.as_ref())?))
    }

    /// Constructs a new `ViewApi` client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new `ViewApi` instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Sends a view request using the `ViewApi` client.
    ///
    /// # Arguments
    ///
    /// * `image` - An `Image` struct containing the information about the image to view.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<u8>` representation of the image on
    /// success, or an error if the request failed.
    pub async fn get(&self, image: &Image) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(self.endpoint.clone())
            .query(&image)
            .send()
            .await?;
        if response.status().is_success() {
            return Ok(response
                .bytes()
                .await
                .map_err(ViewApiError::GetBytesFailed)?
                .to_vec());
        }
        let status = response.status();
        let text = response.text().await.map_err(ViewApiError::GetTextFailed)?;
        Err(ViewApiError::ViewImageFailed {
            status,
            error: text,
        })
    }
}
