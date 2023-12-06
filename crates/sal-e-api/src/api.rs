use anyhow::Context;
use async_trait::async_trait;
use comfyui_api::{comfy::getter::*, models::AsAny};
use dyn_clone::DynClone;
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

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
                prompt,
                count: 1,
                image: None,
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

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams>;
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn img2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response>;

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let base_prompt = config.as_any().downcast_ref().unwrap_or(&self.params);

        let mut prompt = base_prompt.prompt.clone();

        if let Ok(-1) = prompt.seed() {
            *prompt.seed_mut().unwrap() = rand::random::<i64>().abs();
        }

        let images = self.client.execute_prompt(&prompt).await?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(prompt),
            gen_params: Box::new(base_prompt.clone()),
        })
    }

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams> {
        Box::new(self.params.clone())
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

        let mut prompt = base_prompt.prompt.clone();

        *LoadImageExt::image_mut(&mut prompt)? = resp.name;

        if let Ok(-1) = &prompt.seed() {
            *prompt.seed_mut().unwrap() = rand::random::<i64>().abs();
        }

        let images = self.client.execute_prompt(&prompt).await?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(prompt.clone()),
            gen_params: Box::new(base_prompt.clone()),
        })
    }

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams> {
        Box::new(self.params.clone())
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
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.txt2img_defaults);
        let txt2img = self.client.txt2img()?;
        let resp = txt2img.send(config).await?;
        let params = Box::new(resp.info()?);
        Ok(Response {
            images: resp.images()?,
            params: params.clone(),
            gen_params: Box::new(resp.parameters.clone()),
        })
    }

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams> {
        Box::new(self.txt2img_defaults.clone())
    }
}

#[async_trait]
impl Img2ImgApi for StableDiffusionWebUiApi {
    async fn img2img(&self, config: &dyn crate::gen_params::GenParams) -> anyhow::Result<Response> {
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.img2img_defaults);
        let img2img = self.client.img2img()?;
        let resp = img2img.send(config).await?;
        let params = Box::new(resp.info()?);
        Ok(Response {
            images: resp.images()?,
            params: params.clone(),
            gen_params: Box::new(resp.parameters.clone()),
        })
    }

    fn gen_params(&self) -> Box<dyn crate::gen_params::GenParams> {
        Box::new(self.img2img_defaults.clone())
    }
}
