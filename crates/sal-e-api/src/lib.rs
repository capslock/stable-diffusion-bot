use async_trait::async_trait;
use comfyui_api::{comfy::PromptBuilder, models::AsAny};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

#[derive(Debug, Clone, Default)]
pub struct Image {
    pub data: Vec<u8>,
}

pub trait GenParams: std::fmt::Debug + AsAny + Send + Sync {}

impl GenParams for comfyui_api::models::Prompt {}

#[derive(Debug, Clone, Default)]
pub struct ComfyPromptApi {
    pub client: comfyui_api::comfy::Comfy,
    pub prompt: comfyui_api::models::Prompt,
    pub output_node: Option<String>,
    pub prompt_node: Option<String>,
}

impl ComfyPromptApi {
    pub fn new(prompt: comfyui_api::models::Prompt) -> anyhow::Result<Self> {
        Ok(Self {
            client: comfyui_api::comfy::Comfy::new()?,
            prompt,
            ..Default::default()
        })
    }
}

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Txt2ImgApi: Default {
    async fn txt2img(&self, config: &mut dyn GenParams, prompt: &str)
        -> anyhow::Result<Vec<Image>>;
}

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: Default {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        image: &Image,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(
        &self,
        config: &mut dyn GenParams,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>> {
        let defaults = &mut self.prompt.clone();
        let config = config.as_any_mut().downcast_mut().unwrap_or(defaults);
        let prompt = PromptBuilder::new(config, self.output_node.clone())
            .prompt(prompt.to_string(), self.prompt_node.clone())
            .build()?;
        let images = self.client.execute_prompt(&prompt).await?;
        Ok(images
            .into_iter()
            .map(|image| Image { data: image.image })
            .collect())
    }
}

#[async_trait]
impl Img2ImgApi for ComfyPromptApi {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        _image: &Image,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>> {
        let defaults = &mut self.prompt.clone();
        let config = config.as_any_mut().downcast_mut().unwrap_or(defaults);
        let prompt = PromptBuilder::new(config, self.output_node.clone())
            .prompt(prompt.to_string(), self.prompt_node.clone())
            .build()?;
        let images = self.client.execute_prompt(&prompt).await?;
        Ok(images
            .into_iter()
            .map(|image| Image { data: image.image })
            .collect())
    }
}

impl GenParams for Txt2ImgRequest {}
impl GenParams for Img2ImgRequest {}

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
    async fn txt2img(
        &self,
        config: &mut dyn GenParams,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>> {
        let defaults = &mut self.txt2img_defaults.clone();
        let config = config.as_any_mut().downcast_mut().unwrap_or(defaults);
        let txt2img = self.client.txt2img()?;
        let resp = txt2img.send(config.with_prompt(prompt.to_string())).await?;
        Ok(resp
            .images()?
            .into_iter()
            .map(|image| Image { data: image })
            .collect())
    }
}

#[async_trait]
impl Img2ImgApi for StableDiffusionWebUiApi {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        image: &Image,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>> {
        let defaults = &mut self.img2img_defaults.clone();
        let config = config.as_any_mut().downcast_mut().unwrap_or(defaults);
        let img2img = self.client.img2img()?;
        let resp = img2img
            .send(
                config
                    .with_prompt(prompt.to_string())
                    .with_image(image.data.clone()),
            )
            .await?;
        Ok(resp
            .images()?
            .into_iter()
            .map(|image| Image { data: image })
            .collect())
    }
}
