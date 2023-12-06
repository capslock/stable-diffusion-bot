use anyhow::Context;
use reqwest::{multipart, Url};
use serde::{Deserialize, Serialize};

/// Struct representing an image.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUpload {
    /// The filename of the image.
    pub name: String,
    /// The subfolder.
    pub subfolder: String,
    /// The folder type.
    #[serde(rename = "type")]
    pub folder_type: String,
}

/// Struct representing a connection to the ComfyUI API `upload` endpoint.
#[derive(Clone, Debug)]
pub struct UploadApi {
    client: reqwest::Client,
    endpoint: Url,
}

impl UploadApi {
    /// Constructs a new `UploadApi` client with a given `reqwest::Client` and ComfyUI API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `UploadApi` instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new `UploadApi` client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new `UploadApi` instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Uploads an image using the `UploadApi` client.
    ///
    /// # Arguments
    ///
    /// * `image` - A `Vec<u8>` containing the image to upload.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `Image` struct containing information about the image.
    /// success, or an error if the request failed.
    pub async fn image(&self, image: Vec<u8>) -> anyhow::Result<ImageUpload> {
        let file = multipart::Part::bytes(image)
            .file_name("image.png")
            .mime_str("image/png")?;
        let form = multipart::Form::new().part("image", file);
        let response = self
            .client
            .post(
                self.endpoint
                    .clone()
                    .join("image")
                    .context("failed to parse url")?,
            )
            .multipart(form)
            .send()
            .await
            .context("failed to send request")?;
        if response.status().is_success() {
            return response.json().await.context("failed to get response json");
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
