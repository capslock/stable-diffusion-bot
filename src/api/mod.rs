mod txt2img;
use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
pub use txt2img::*;

mod img2img;
pub use img2img::*;

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
            url: Url::parse("http://localhost:7860").expect("Failed to parse default URL"),
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

    /// Returns a new instance of `Txt2Img` with the API's cloned `reqwest::Client` and the URL for `txt2img` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn txt2img(&self) -> anyhow::Result<Txt2Img> {
        Ok(Txt2Img::new_with_url(
            self.client.clone(),
            self.url
                .join("sdapi/v1/txt2img")
                .context("Failed to parse txt2img endpoint")?,
        ))
    }

    /// Returns a new instance of `Img2Img` with the API's cloned `reqwest::Client` and the URL for `img2img` endpoint.
    ///
    /// # Errors
    ///
    /// If the URL fails to parse, an error will be returned.
    pub fn img2img(&self) -> anyhow::Result<Img2Img> {
        Ok(Img2Img::new_with_url(
            self.client.clone(),
            self.url
                .join("sdapi/v1/img2img")
                .context("Failed to parse img2img endpoint")?,
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
pub struct ImgInfo {
    pub batch_size: Option<u32>,
    pub all_prompts: Option<Vec<String>>,
    pub styles: Option<Vec<String>>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub extra_generation_params: Option<serde_json::Value>,
    pub sampler_name: Option<String>,
    pub restore_faces: Option<bool>,
    pub seed_resize_from_w: Option<i32>,
    pub all_negative_prompts: Option<Vec<String>>,
    pub cfg_scale: Option<f64>,
    pub index_of_first_image: Option<u32>,
    pub seed_resize_from_h: Option<i32>,
    pub infotexts: Option<Vec<String>>,
    pub negative_prompt: Option<String>,
    pub seed: Option<i64>,
    pub denoising_strength: Option<f64>,
    pub is_using_inpainting_conditioning: Option<bool>,
    pub subseed: Option<i64>,
    pub prompt: Option<String>,
    pub subseed_strength: Option<u32>,
    pub all_subseeds: Option<Vec<i64>>,
    pub steps: Option<u32>,
    pub face_restoration_model: Option<serde_json::Value>,
    pub job_timestamp: Option<String>,
    pub clip_skip: Option<u32>,
    pub sd_model_hash: Option<String>,
    pub all_seeds: Option<Vec<i64>>,
}
