use reqwest::{multipart, Url};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum UploadApiError {
    /// Error parsing endpoint URL
    #[error("Failed to parse endpoint URL")]
    ParseError(#[from] url::ParseError),
    /// Error setting MIME type
    #[error("Failed to set MIME type")]
    SetMimeStrFailed(#[source] reqwest::Error),
    /// Error sending request
    #[error("Failed to send request")]
    RequestFailed(#[from] reqwest::Error),
    /// An error occurred while parsing the response from the API.
    #[error("Parsing response failed")]
    InvalidResponse(#[source] reqwest::Error),
    /// An error occurred getting response data.
    #[error("Failed to get response data")]
    GetDataFailed(#[source] reqwest::Error),
    /// Server returned an error when uploading file
    #[error("Failed to upload image: {status}: {error}")]
    UploadImageFailed {
        status: reqwest::StatusCode,
        error: String,
    },
}

type Result<T> = std::result::Result<T, UploadApiError>;

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
    /// endpoint.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `str` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `UploadApi` instance on success, or an error if url parsing failed.
    pub fn new<S>(client: reqwest::Client, endpoint: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self::new_with_url(client, Url::parse(endpoint.as_ref())?))
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
    pub async fn image(&self, image: Vec<u8>) -> Result<ImageUpload> {
        let file = multipart::Part::bytes(image)
            .file_name("image.png")
            .mime_str("image/png")
            .map_err(UploadApiError::SetMimeStrFailed)?;
        let form = multipart::Form::new().part("image", file);
        let response = self
            .client
            .post(self.endpoint.clone().join("image")?)
            .multipart(form)
            .send()
            .await
            .map_err(UploadApiError::RequestFailed)?;
        if response.status().is_success() {
            return response
                .json()
                .await
                .map_err(UploadApiError::InvalidResponse);
        }
        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(UploadApiError::GetDataFailed)?;
        Err(UploadApiError::UploadImageFailed {
            status,
            error: text,
        })
    }
}
