use anyhow::Context;
use async_trait::async_trait;
use comfyui_api::{comfy::getter::*, models::AsAny};
use dyn_clone::DynClone;
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

use crate::{ComfyParams, Img2ImgParams, Txt2ImgParams};

/// Struct representing a response from a Stable Diffusion API image generation endpoint.
#[derive(Debug, Clone)]
pub struct Response {
    /// A vector of images.
    pub images: Vec<Vec<u8>>,
    /// The parameters describing the generated image.
    pub params: Box<dyn crate::image_params::ImageParams>,
    /// The parameters that were provided for the generation request.
    pub gen_params: Box<dyn crate::gen_params::GenParams>,
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ComfyPromptApiError {
    /// Error creating a ComfyUI Client
    #[error("Error creating a ComfyUI Client")]
    CreateClient(#[from] comfyui_api::comfy::ComfyApiError),
}

/// Struct wrapping a connection to the ComfyUI API.
#[derive(Debug, Clone, Default)]
pub struct ComfyPromptApi {
    /// The ComfyUI client.
    pub client: comfyui_api::comfy::Comfy,
    /// Default parameters for the ComfyUI API.
    pub params: crate::gen_params::ComfyParams,
    /// The output node.
    pub output_node: Option<String>,
    /// The prompt node.
    pub prompt_node: Option<String>,
}

impl ComfyPromptApi {
    /// Constructs a new `ComfyPromptApi` client with the provided prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to use for the API.
    ///
    /// # Returns
    ///
    /// A new `ComfyPromptApi` instance on success, or an error if there was a failure in the ComfyUI API client.
    pub fn new(prompt: comfyui_api::models::Prompt) -> Result<Self, ComfyPromptApiError> {
        Ok(Self {
            client: comfyui_api::comfy::Comfy::new()?,
            params: crate::gen_params::ComfyParams {
                prompt: Some(prompt),
                count: 1,
                seed: Some(-1),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    /// Constructs a new `ComfyPromptApi` client with the provided url and prompt.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to use for the API. Must be a valid URL, e.g. `http://localhost:8188
    /// * `prompt` - The prompt to use for the API.
    ///
    /// # Returns
    ///
    /// A new `ComfyPromptApi` instance on success, or an error if there was a failure in the ComfyUI API client.
    pub fn new_with_url<S>(
        url: S,
        prompt: comfyui_api::models::Prompt,
    ) -> Result<Self, ComfyPromptApiError>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            client: comfyui_api::comfy::Comfy::new_with_url(url)?,
            params: crate::gen_params::ComfyParams {
                prompt: Some(prompt),
                count: 1,
                seed: Some(-1),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    /// Constructs a new `ComfyPromptApi` client with the provided url and prompt.
    ///
    /// # Arguments
    ///
    /// * `client` - An instance of `reqwest::Client`.
    /// * `url` - The URL to use for the API. Must be a valid URL, e.g. `http://localhost:8188
    /// * `prompt` - The prompt to use for the API.
    ///
    /// # Returns
    ///
    /// A new `ComfyPromptApi` instance on success, or an error if there was a failure in the ComfyUI API client.
    pub fn new_with_client_and_url<S>(
        client: reqwest::Client,
        url: S,
        prompt: comfyui_api::models::Prompt,
    ) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            client: comfyui_api::comfy::Comfy::new_with_client_and_url(client, url)?,
            params: crate::gen_params::ComfyParams {
                prompt: Some(prompt),
                count: 1,
                seed: Some(-1),
                ..Default::default()
            },
            ..Default::default()
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Txt2ImgApiError {
    /// Prompt was empty.
    #[error("Prompt was empty.")]
    EmptyPrompt,
    /// Error running txt2img.
    #[error("Error running txt2img.")]
    Txt2Img(#[from] anyhow::Error),
    /// Error parsing response.
    #[error("Error parsing response.")]
    ParseResponse(#[source] anyhow::Error),
}

dyn_clone::clone_trait_object!(Txt2ImgApi);

/// Trait representing a Txt2Img endpoint.
#[async_trait]
pub trait Txt2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    /// Generates an image using text-to-image.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for the generation.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Response` on success, or an error if the request failed.
    async fn txt2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Txt2ImgApiError>;

    /// Returns the default generation parameters for this endpoint.
    ///
    /// # Arguments
    ///
    /// * `user_settings` - The user settings to merge with the defaults.
    ///
    /// # Returns
    ///
    /// A `Box<dyn crate::gen_params::GenParams>` containing the generation parameters.
    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams>;
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Img2ImgApiError {
    /// Prompt was empty.
    #[error("Prompt was empty.")]
    EmptyPrompt,
    /// Error running txt2img.
    #[error("Error running img2img.")]
    Img2Img(#[from] anyhow::Error),
    /// Error parsing response.
    #[error("Error parsing response.")]
    ParseResponse(#[source] anyhow::Error),
    /// No image provided.
    #[error("No image provided.")]
    NoImage,
    /// Error uploading image.
    #[error("Error uploading image.")]
    UploadImage(#[source] anyhow::Error),
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Trait representing an Img2Img endpoint.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    /// Generates an image using image-to-image.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to use for the generation.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Response` on success, or an error if the request failed.
    async fn img2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Img2ImgApiError>;

    /// Returns the default generation parameters for this endpoint.
    ///
    /// # Arguments
    ///
    /// * `user_settings` - The user settings to merge with the defaults.
    ///
    /// # Returns
    ///
    /// A `Box<dyn crate::gen_params::GenParams>` containing the generation parameters.
    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Txt2ImgApiError> {
        let base_prompt = config.as_any().downcast_ref().unwrap_or(&self.params);

        let mut new_prompt = base_prompt.clone();
        if let Some(-1) = new_prompt.seed {
            new_prompt.seed = Some(rand::random::<i64>().abs());
        }

        let prompt = new_prompt.apply().context(Txt2ImgApiError::EmptyPrompt)?;

        let images = self
            .client
            .execute_prompt(&prompt)
            .await
            .context("Failed to execute prompt")?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(prompt),
            gen_params: Box::new(base_prompt.clone()),
        })
    }

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams> {
        if let Some(user_settings) = user_settings {
            let mut params = ComfyParams::from(user_settings);
            params.prompt = self.params.prompt.clone();
            Box::new(params)
        } else {
            Box::new(self.params.clone())
        }
    }
}

#[async_trait]
impl Img2ImgApi for ComfyPromptApi {
    async fn img2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Img2ImgApiError> {
        let base_prompt = config.as_any().downcast_ref().unwrap_or(&self.params);

        let resp = if let Some(image) = &base_prompt.image {
            self.client
                .upload_file(image.clone())
                .await
                .context("Failed to upload image")
                .map_err(Img2ImgApiError::UploadImage)?
        } else {
            return Err(Img2ImgApiError::NoImage);
        };

        let mut new_prompt = base_prompt.clone();
        if let Some(-1) = new_prompt.seed {
            new_prompt.seed = Some(rand::random::<i64>().abs());
        }

        let mut prompt = new_prompt.apply().context(Img2ImgApiError::EmptyPrompt)?;

        *prompt.image_mut()? = resp.name;

        let images = self
            .client
            .execute_prompt(&prompt)
            .await
            .context("Failed to execute prompt")?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(prompt.clone()),
            gen_params: Box::new(base_prompt.clone()),
        })
    }

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams> {
        if let Some(user_settings) = user_settings {
            let mut params = ComfyParams::from(user_settings);
            params.prompt = self.params.prompt.clone();
            Box::new(params)
        } else {
            Box::new(self.params.clone())
        }
    }
}

/// Struct wrapping a connection to the Stable Diffusion WebUI API.
#[derive(Debug, Clone, Default)]
pub struct StableDiffusionWebUiApi {
    /// The Stable Diffusion WebUI client.
    pub client: stable_diffusion_api::Api,
    /// Default parameters for the Txt2Img endpoint.
    pub txt2img_defaults: Txt2ImgRequest,
    /// Default parameters for the Img2Img endpoint.
    pub img2img_defaults: Img2ImgRequest,
}

impl StableDiffusionWebUiApi {
    /// Constructs a new `StableDiffusionWebUiApi` client with the default parameters.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Txt2ImgApi for StableDiffusionWebUiApi {
    async fn txt2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Txt2ImgApiError> {
        let config = Txt2ImgParams::from(config);
        let txt2img = self
            .client
            .txt2img()
            .context("Failed to open txt2img API")?;
        let resp = txt2img
            .send(&config.user_params)
            .await
            .context("Failed to send request")?;
        let params = Box::new(
            resp.info()
                .context("Failed to parse info from response")
                .map_err(Txt2ImgApiError::ParseResponse)?,
        );
        Ok(Response {
            images: resp
                .images()
                .context("Failed to parse image from response")
                .map_err(Txt2ImgApiError::ParseResponse)?,
            params: params.clone(),
            gen_params: Box::new(Txt2ImgParams {
                user_params: resp.parameters.clone(),
                defaults: Some(self.txt2img_defaults.clone()),
            }),
        })
    }

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams> {
        if let Some(user_settings) = user_settings {
            Box::new(Txt2ImgParams {
                user_params: Txt2ImgParams::from(user_settings).user_params,
                defaults: Some(self.txt2img_defaults.clone()),
            })
        } else {
            Box::new(Txt2ImgParams {
                user_params: Txt2ImgRequest::default(),
                defaults: Some(self.txt2img_defaults.clone()),
            })
        }
    }
}

#[async_trait]
impl Img2ImgApi for StableDiffusionWebUiApi {
    async fn img2img(
        &self,
        config: &dyn crate::gen_params::GenParams,
    ) -> Result<Response, Img2ImgApiError> {
        let config = Img2ImgParams::from(config);
        let img2img = self
            .client
            .img2img()
            .context("Failed to open img2img API")?;
        let resp = img2img
            .send(&config.user_params)
            .await
            .context("Failed to send request")?;
        let params = Box::new(
            resp.info()
                .context("Failed to parse info from response")
                .map_err(Img2ImgApiError::ParseResponse)?,
        );
        Ok(Response {
            images: resp
                .images()
                .context("Failed to parse image from response")
                .map_err(Img2ImgApiError::ParseResponse)?,
            params: params.clone(),
            gen_params: Box::new(Img2ImgParams {
                user_params: resp.parameters.clone(),
                defaults: Some(self.img2img_defaults.clone()),
            }),
        })
    }

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams> {
        if let Some(user_settings) = user_settings {
            Box::new(Txt2ImgParams {
                user_params: Txt2ImgParams::from(user_settings).user_params,
                defaults: Some(self.txt2img_defaults.clone()),
            })
        } else {
            Box::new(Txt2ImgParams {
                user_params: Txt2ImgRequest::default(),
                defaults: Some(self.txt2img_defaults.clone()),
            })
        }
    }
}
