use async_trait::async_trait;
use comfyui_api::{comfy::getter::*, comfy::PromptBuilder, models::AsAny};
use dyn_clone::DynClone;
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

#[derive(Debug, Clone, Default)]
pub struct Image {
    pub data: Vec<u8>,
}

dyn_clone::clone_trait_object!(GenParams);

#[typetag::serde]
pub trait GenParams: std::fmt::Debug + AsAny + Send + Sync + DynClone {
    fn seed(&self) -> Option<i64>;
    fn set_seed(&mut self, seed: i64);

    fn steps(&self) -> Option<u32>;
    fn set_steps(&mut self, steps: u32);

    fn count(&self) -> Option<u32>;
    fn set_count(&mut self, count: u32);

    fn cfg(&self) -> Option<f32>;
    fn set_cfg(&mut self, cfg: f32);

    fn width(&self) -> Option<u32>;
    fn set_width(&mut self, width: u32);

    fn height(&self) -> Option<u32>;
    fn set_height(&mut self, height: u32);

    fn prompt(&self) -> Option<String>;
    fn set_prompt(&mut self, prompt: String);

    fn negative_prompt(&self) -> Option<String>;
    fn set_negative_prompt(&mut self, negative_prompt: String);

    fn denoising(&self) -> Option<f32>;
    fn set_denoising(&mut self, denoising: f32);
}

#[typetag::serde]
impl GenParams for comfyui_api::models::Prompt {
    fn seed(&self) -> Option<i64> {
        (self as &dyn SeedExt).seed().ok()
    }

    fn set_seed(&mut self, seed: i64) {
        (self as &mut dyn SeedExt).set_seed(seed).ok();
    }

    fn steps(&self) -> Option<u32> {
        unimplemented!()
    }

    fn set_steps(&mut self, steps: u32) {
        unimplemented!()
    }

    fn count(&self) -> Option<u32> {
        unimplemented!()
    }

    fn set_count(&mut self, count: u32) {
        unimplemented!()
    }

    fn cfg(&self) -> Option<f32> {
        unimplemented!()
    }

    fn set_cfg(&mut self, cfg: f32) {
        unimplemented!()
    }

    fn width(&self) -> Option<u32> {
        (self as &dyn WidthExt).width().ok()
    }

    fn set_width(&mut self, width: u32) {
        (self as &mut dyn WidthExt).set_width(width).ok();
    }

    fn height(&self) -> Option<u32> {
        (self as &dyn HeightExt).height().ok()
    }

    fn set_height(&mut self, height: u32) {
        (self as &mut dyn HeightExt).set_height(height).ok();
    }

    fn prompt(&self) -> Option<String> {
        (self as &dyn PromptExt).prompt().ok().cloned()
    }

    fn set_prompt(&mut self, prompt: String) {
        if let Ok(p) = (self as &mut dyn PromptExt).prompt_mut() {
            *p = prompt;
        }
    }

    fn negative_prompt(&self) -> Option<String> {
        (self as &dyn NegativePromptExt)
            .negative_prompt()
            .ok()
            .cloned()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        if let Ok(p) = (self as &mut dyn NegativePromptExt).negative_prompt_mut() {
            *p = negative_prompt;
        }
    }

    fn denoising(&self) -> Option<f32> {
        unimplemented!()
    }

    fn set_denoising(&mut self, denoising: f32) {
        unimplemented!()
    }
}

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

dyn_clone::clone_trait_object!(Txt2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Txt2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn txt2img(&self, config: &mut dyn GenParams, prompt: &str)
        -> anyhow::Result<Vec<Image>>;

    fn gen_params(&self) -> Box<dyn GenParams>;
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        image: Vec<u8>,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>>;

    fn gen_params(&self) -> Box<dyn GenParams>;
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

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.prompt.clone())
    }
}

#[async_trait]
impl Img2ImgApi for ComfyPromptApi {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        _image: Vec<u8>,
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

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.prompt.clone())
    }
}

#[typetag::serde]
impl GenParams for Txt2ImgRequest {
    fn seed(&self) -> Option<i64> {
        todo!()
    }

    fn set_seed(&mut self, seed: i64) {
        todo!()
    }

    fn steps(&self) -> Option<u32> {
        todo!()
    }

    fn set_steps(&mut self, steps: u32) {
        todo!()
    }

    fn count(&self) -> Option<u32> {
        todo!()
    }

    fn set_count(&mut self, count: u32) {
        todo!()
    }

    fn cfg(&self) -> Option<f32> {
        todo!()
    }

    fn set_cfg(&mut self, cfg: f32) {
        todo!()
    }

    fn width(&self) -> Option<u32> {
        todo!()
    }

    fn set_width(&mut self, width: u32) {
        todo!()
    }

    fn height(&self) -> Option<u32> {
        todo!()
    }

    fn set_height(&mut self, height: u32) {
        todo!()
    }

    fn prompt(&self) -> Option<String> {
        todo!()
    }

    fn set_prompt(&mut self, prompt: String) {
        todo!()
    }

    fn negative_prompt(&self) -> Option<String> {
        todo!()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        todo!()
    }

    fn denoising(&self) -> Option<f32> {
        todo!()
    }

    fn set_denoising(&mut self, denoising: f32) {
        todo!()
    }
}

#[typetag::serde]
impl GenParams for Img2ImgRequest {
    fn seed(&self) -> Option<i64> {
        todo!()
    }

    fn set_seed(&mut self, seed: i64) {
        todo!()
    }

    fn steps(&self) -> Option<u32> {
        todo!()
    }

    fn set_steps(&mut self, steps: u32) {
        todo!()
    }

    fn count(&self) -> Option<u32> {
        todo!()
    }

    fn set_count(&mut self, count: u32) {
        todo!()
    }

    fn cfg(&self) -> Option<f32> {
        todo!()
    }

    fn set_cfg(&mut self, cfg: f32) {
        todo!()
    }

    fn width(&self) -> Option<u32> {
        todo!()
    }

    fn set_width(&mut self, width: u32) {
        todo!()
    }

    fn height(&self) -> Option<u32> {
        todo!()
    }

    fn set_height(&mut self, height: u32) {
        todo!()
    }

    fn prompt(&self) -> Option<String> {
        todo!()
    }

    fn set_prompt(&mut self, prompt: String) {
        todo!()
    }

    fn negative_prompt(&self) -> Option<String> {
        todo!()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        todo!()
    }

    fn denoising(&self) -> Option<f32> {
        todo!()
    }

    fn set_denoising(&mut self, denoising: f32) {
        todo!()
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

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.txt2img_defaults.clone())
    }
}

#[async_trait]
impl Img2ImgApi for StableDiffusionWebUiApi {
    async fn img2img(
        &self,
        config: &mut dyn GenParams,
        image: Vec<u8>,
        prompt: &str,
    ) -> anyhow::Result<Vec<Image>> {
        let defaults = &mut self.img2img_defaults.clone();
        let config = config.as_any_mut().downcast_mut().unwrap_or(defaults);
        let img2img = self.client.img2img()?;
        let resp = img2img
            .send(config.with_prompt(prompt.to_string()).with_image(image))
            .await?;
        Ok(resp
            .images()?
            .into_iter()
            .map(|image| Image { data: image })
            .collect())
    }

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.img2img_defaults.clone())
    }
}
