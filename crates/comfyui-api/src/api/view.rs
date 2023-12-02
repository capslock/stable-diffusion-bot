use anyhow::Context;
use reqwest::Url;

use crate::models::Image;

/// Struct representing a connection to the ComfyUI API `view` endpoint.
#[derive(Clone, Debug)]
pub struct ViewApi {
    client: reqwest::Client,
    endpoint: Url,
}

impl ViewApi {
    /// Constructs a new `ViewApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `ViewApi` instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
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
    pub async fn get(&self, image: &Image) -> anyhow::Result<Vec<u8>> {
        let response = self
            .client
            .get(self.endpoint.clone())
            .query(&image)
            .send()
            .await
            .context("failed to send request")?;
        if response.status().is_success() {
            return Ok(response
                .bytes()
                .await
                .context("failed to get bytes")?
                .to_vec());
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
