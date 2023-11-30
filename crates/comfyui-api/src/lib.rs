use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod prompt;
pub use prompt::*;

/// Struct representing a connection to a Stable Diffusion WebUI API.
#[derive(Clone, Debug)]
pub struct Api {
    client: reqwest::Client,
    url: Url,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            url: Url::parse("http://localhost:8188").expect("Failed to parse default URL"),
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
    /// * `url` - A string that specifies the Stable Diffusion WebUI API URL endpoint.
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
    /// * `url` - A string that specifies the Stable Diffusion WebUI API URL endpoint.
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
        })
    }

    pub fn prompt(&self) -> anyhow::Result<Comfy> {
        Ok(Comfy::new_with_url(
            self.client.clone(),
            self.url
                .join("prompt")
                .context("Failed to parse comfyUI endpoint")?,
        ))
    }
}

/// A struct that represents the response from the Stable Diffusion WebUI API endpoint.
#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct ImgResponse<T> {
    /// A vector of strings containing base64-encoded images.
    pub images: Vec<String>,
    /// The parameters that were provided for the generation request.
    pub parameters: T,
    /// A string containing JSON representing information about the request.
    pub info: String,
}

impl<T> ImgResponse<T> {
    /// Parses and returns a new `ImgInfo` instance from the `info` field of the `ImgResponse`.
    ///
    /// # Errors
    ///
    /// If the `info` field fails to parse, an error will be returned.
    pub fn info(&self) -> anyhow::Result<ImgInfo> {
        serde_json::from_str(&self.info).context("failed to parse info")
    }
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize, Debug)]
/// Information about the generated images.
pub struct ImgInfo {
    /// The prompt used when generating the image.
    pub prompt: Option<String>,
    /// A vector of all the prompts used for image generation.
    pub all_prompts: Option<Vec<String>>,
    /// The negative prompt used when generating the image.
    pub negative_prompt: Option<String>,
    /// A vector of all negative prompts used when generating the image.
    pub all_negative_prompts: Option<Vec<String>>,
    /// The random seed used for image generation.
    pub seed: Option<i64>,
    /// A vector of all the random seeds used for image generation.
    pub all_seeds: Option<Vec<i64>>,
    /// The subseed used when generating the image.
    pub subseed: Option<i64>,
    /// A vector of all the subseeds used for image generation.
    pub all_subseeds: Option<Vec<i64>>,
    /// The strength of the subseed used when generating the image.
    pub subseed_strength: Option<u32>,
    /// The width of the generated image.
    pub width: Option<i32>,
    /// The height of the generated image.
    pub height: Option<i32>,
    /// The name of the sampler used for image generation.
    pub sampler_name: Option<String>,
    /// The cfg scale factor used when generating the image.
    pub cfg_scale: Option<f64>,
    /// The number of steps taken when generating the image.
    pub steps: Option<u32>,
    /// The number of images generated in one batch.
    pub batch_size: Option<u32>,
    /// Whether or not face restoration was used.
    pub restore_faces: Option<bool>,
    /// The face restoration model used when generating the image.
    pub face_restoration_model: Option<serde_json::Value>,
    /// The name of the sd model used when generating the image.
    pub sd_model_name: Option<String>,
    /// The hash of the sd model used for image generation.
    pub sd_model_hash: Option<String>,
    /// The name of the VAE used when generating the image.
    pub sd_vae_name: Option<String>,
    /// The hash of the VAE used for image generation.
    pub sd_vae_hash: Option<String>,
    /// The width used when resizing the image seed.
    pub seed_resize_from_w: Option<i32>,
    /// The height used when resizing the image seed.
    pub seed_resize_from_h: Option<i32>,
    /// The strength of the denoising applied during image generation.
    pub denoising_strength: Option<f64>,
    /// Extra parameters passed for image generation.
    pub extra_generation_params: Option<ExtraGenParams>,
    /// The index of the first image.
    pub index_of_first_image: Option<u32>,
    /// A vector of information texts about the generated images.
    pub infotexts: Option<Vec<String>>,
    /// A vector of the styles used for image generation.
    pub styles: Option<Vec<String>>,
    /// The timestamp of when the job was started.
    pub job_timestamp: Option<String>,
    /// The number of clip layers skipped during image generation.
    pub clip_skip: Option<u32>,
    /// Whether or not inpainting conditioning was used for image generation.
    pub is_using_inpainting_conditioning: Option<bool>,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize, Debug)]
/// Extra parameters describing image generation.
pub struct ExtraGenParams {
    /// Names and hashes of LORA models used for image generation.
    #[serde(rename = "Lora hashes")]
    pub lora_hashes: Option<String>,
    /// Names and hashes of Textual Inversion models used for image generation.
    #[serde(rename = "TI hashes")]
    pub ti_hashes: Option<String>,
}

pub struct Comfy {
    client: reqwest::Client,
    endpoint: Url,
}

impl Comfy {
    /// Constructs a new Txt2Img client with a given `reqwest::Client` and Stable Diffusion API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new Txt2Img instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new Txt2Img client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new Txt2Img instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Sends an image request using the Txt2Img client.
    ///
    /// # Arguments
    ///
    /// * `request` - An Txt2ImgRequest containing the parameters for the image request.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ImgResponse<Txt2ImgRequest>` on success, or an error if one occurred.
    pub async fn send(&self, request: &Prompt) -> anyhow::Result<()> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&request)
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
