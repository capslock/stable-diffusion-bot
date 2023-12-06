use anyhow::Context;
use async_trait::async_trait;
use comfyui_api::{comfy::getter::*, models::AsAny};
use dyn_clone::DynClone;
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

use crate::{ComfyParams, Img2ImgParams, Txt2ImgParams};

#[derive(Debug, Clone)]
pub struct Response {
    pub images: Vec<Vec<u8>>,
    pub params: Box<dyn crate::image_params::ImageParams>,
    pub gen_params: Box<dyn crate::gen_params::GenParams>,
}

#[derive(Debug, Clone, Default)]
pub struct ComfyPromptApi {
    pub client: comfyui_api::comfy::Comfy,
    pub params: crate::gen_params::ComfyParams,
    pub output_node: Option<String>,
    pub prompt_node: Option<String>,
}

impl ComfyPromptApi {
    pub fn new(prompt: comfyui_api::models::Prompt) -> anyhow::Result<Self> {
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
}

dyn_clone::clone_trait_object!(Txt2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Txt2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn txt2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response>;

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams>;
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn img2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response>;

    fn gen_params(
        &self,
        user_settings: Option<&dyn crate::gen_params::GenParams>,
    ) -> Box<dyn crate::gen_params::GenParams>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let base_prompt = config.as_any().downcast_ref().unwrap_or(&self.params);

        let mut new_prompt = base_prompt.clone();
        if let Some(-1) = new_prompt.seed {
            new_prompt.seed = Some(rand::random::<i64>().abs());
        }

        let prompt = new_prompt.apply().context("Prompt was empty.")?;

        let images = self.client.execute_prompt(&prompt).await?;
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
    async fn img2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let base_prompt = config.as_any().downcast_ref().unwrap_or(&self.params);

        let resp = if let Some(image) = &base_prompt.image {
            self.client
                .upload_file(image.clone())
                .await
                .context("Failed to upload file")?
        } else {
            return Err(anyhow::anyhow!("Required image was not provided"));
        };

        let mut new_prompt = base_prompt.clone();
        if let Some(-1) = new_prompt.seed {
            new_prompt.seed = Some(rand::random::<i64>().abs());
        }

        let mut prompt = new_prompt.apply().context("Prompt was empty.")?;

        *prompt.image_mut()? = resp.name;

        let images = self.client.execute_prompt(&prompt).await?;
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

#[derive(Debug, Clone, Default)]
pub struct StableDiffusionWebUiApi {
    pub client: stable_diffusion_api::Api,
    pub txt2img_defaults: Txt2ImgRequest,
    pub img2img_defaults: Img2ImgRequest,
}

impl StableDiffusionWebUiApi {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Txt2ImgApi for StableDiffusionWebUiApi {
    async fn txt2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let config = Txt2ImgParams::from(config);
        let txt2img = self.client.txt2img()?;
        let resp = txt2img.send(&config.user_params).await?;
        let params = Box::new(resp.info()?);
        Ok(Response {
            images: resp.images()?,
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
    async fn img2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let config = Img2ImgParams::from(config);
        let img2img = self.client.img2img()?;
        let resp = img2img.send(&config.user_params).await?;
        let params = Box::new(resp.info()?);
        Ok(Response {
            images: resp.images()?,
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
