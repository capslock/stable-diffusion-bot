use async_trait::async_trait;
use comfyui_api::{comfy::getter::*, models::AsAny};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComfyParams {
    pub prompt: comfyui_api::models::Prompt,
    pub count: u32,
}

#[typetag::serde]
impl GenParams for ComfyParams {
    fn seed(&self) -> Option<i64> {
        (&self.prompt as &dyn SeedExt).seed().ok().copied()
    }

    fn set_seed(&mut self, seed: i64) {
        if let Ok(value) = (&mut self.prompt as &mut dyn SeedExt).seed_mut() {
            *value = seed;
        }
    }

    fn steps(&self) -> Option<u32> {
        (&self.prompt as &dyn StepsExt).steps().ok().copied()
    }

    fn set_steps(&mut self, steps: u32) {
        if let Ok(value) = (&mut self.prompt as &mut dyn StepsExt).steps_mut() {
            *value = steps;
        }
    }

    fn count(&self) -> Option<u32> {
        Some(self.count)
    }

    fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    fn cfg(&self) -> Option<f32> {
        (&self.prompt as &dyn CfgExt).cfg().ok().copied()
    }

    fn set_cfg(&mut self, cfg: f32) {
        if let Ok(value) = (&mut self.prompt as &mut dyn CfgExt).cfg_mut() {
            *value = cfg;
        }
    }

    fn width(&self) -> Option<u32> {
        (&self.prompt as &dyn WidthExt).width().ok().copied()
    }

    fn set_width(&mut self, width: u32) {
        if let Ok(value) = (&mut self.prompt as &mut dyn WidthExt).width_mut() {
            *value = width;
        }
    }

    fn height(&self) -> Option<u32> {
        (&self.prompt as &dyn HeightExt).height().ok().copied()
    }

    fn set_height(&mut self, height: u32) {
        if let Ok(value) = (&mut self.prompt as &mut dyn HeightExt).height_mut() {
            *value = height;
        }
    }

    fn prompt(&self) -> Option<String> {
        (&self.prompt as &dyn PromptExt).prompt().ok().cloned()
    }

    fn set_prompt(&mut self, prompt: String) {
        if let Ok(p) = (&mut self.prompt as &mut dyn PromptExt).prompt_mut() {
            *p = prompt;
        }
    }

    fn negative_prompt(&self) -> Option<String> {
        (&self.prompt as &dyn NegativePromptExt)
            .negative_prompt()
            .ok()
            .cloned()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        if let Ok(p) = (&mut self.prompt as &mut dyn NegativePromptExt).negative_prompt_mut() {
            *p = negative_prompt;
        }
    }

    fn denoising(&self) -> Option<f32> {
        (&self.prompt as &dyn DenoiseExt).denoise().ok().copied()
    }

    fn set_denoising(&mut self, denoising: f32) {
        if let Ok(value) = (&mut self.prompt as &mut dyn DenoiseExt).denoise_mut() {
            *value = denoising;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComfyPromptApi {
    pub client: comfyui_api::comfy::Comfy,
    pub params: ComfyParams,
    pub output_node: Option<String>,
    pub prompt_node: Option<String>,
}

impl ComfyPromptApi {
    pub fn new(prompt: comfyui_api::models::Prompt) -> anyhow::Result<Self> {
        Ok(Self {
            client: comfyui_api::comfy::Comfy::new()?,
            params: ComfyParams { prompt, count: 1 },
            ..Default::default()
        })
    }
}

dyn_clone::clone_trait_object!(Txt2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Txt2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>>;

    fn gen_params(&self) -> Box<dyn GenParams>;
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>>;

    fn gen_params(&self) -> Box<dyn GenParams>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>> {
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.params.prompt);
        let images = self.client.execute_prompt(config).await?;
        Ok(images
            .into_iter()
            .map(|image| Image { data: image.image })
            .collect())
    }

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.params.clone())
    }
}

#[async_trait]
impl Img2ImgApi for ComfyPromptApi {
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>> {
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.params.prompt);
        let images = self.client.execute_prompt(config).await?;
        Ok(images
            .into_iter()
            .map(|image| Image { data: image.image })
            .collect())
    }

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.params.clone())
    }
}

#[typetag::serde]
impl GenParams for Txt2ImgRequest {
    fn seed(&self) -> Option<i64> {
        self.seed
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = Some(seed);
    }

    fn steps(&self) -> Option<u32> {
        self.steps
    }

    fn set_steps(&mut self, steps: u32) {
        self.steps = Some(steps);
    }

    fn count(&self) -> Option<u32> {
        self.n_iter
    }

    fn set_count(&mut self, count: u32) {
        self.n_iter = Some(count);
    }

    fn cfg(&self) -> Option<f32> {
        self.cfg_scale.map(|c| c as f32)
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.cfg_scale = Some(cfg as f64);
    }

    fn width(&self) -> Option<u32> {
        self.width
    }

    fn set_width(&mut self, width: u32) {
        self.width = Some(width);
    }

    fn height(&self) -> Option<u32> {
        self.height
    }

    fn set_height(&mut self, height: u32) {
        self.height = Some(height);
    }

    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn set_prompt(&mut self, prompt: String) {
        self.prompt = Some(prompt);
    }

    fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt.clone()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.negative_prompt = Some(negative_prompt);
    }

    fn denoising(&self) -> Option<f32> {
        self.denoising_strength.map(|d| d as f32)
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.denoising_strength = Some(denoising as f64);
    }
}

#[typetag::serde]
impl GenParams for Img2ImgRequest {
    fn seed(&self) -> Option<i64> {
        self.seed
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = Some(seed);
    }

    fn steps(&self) -> Option<u32> {
        self.steps
    }

    fn set_steps(&mut self, steps: u32) {
        self.steps = Some(steps);
    }

    fn count(&self) -> Option<u32> {
        self.n_iter
    }

    fn set_count(&mut self, count: u32) {
        self.n_iter = Some(count);
    }

    fn cfg(&self) -> Option<f32> {
        self.cfg_scale.map(|c| c as f32)
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.cfg_scale = Some(cfg as f64);
    }

    fn width(&self) -> Option<u32> {
        self.width
    }

    fn set_width(&mut self, width: u32) {
        self.width = Some(width);
    }

    fn height(&self) -> Option<u32> {
        self.height
    }

    fn set_height(&mut self, height: u32) {
        self.height = Some(height);
    }

    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn set_prompt(&mut self, prompt: String) {
        self.prompt = Some(prompt);
    }

    fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt.clone()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.negative_prompt = Some(negative_prompt);
    }

    fn denoising(&self) -> Option<f32> {
        self.denoising_strength.map(|d| d as f32)
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.denoising_strength = Some(denoising as f64);
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
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>> {
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.txt2img_defaults);
        let txt2img = self.client.txt2img()?;
        let resp = txt2img.send(config).await?;
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
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Vec<Image>> {
        let config = config
            .as_any()
            .downcast_ref()
            .unwrap_or(&self.img2img_defaults);
        let img2img = self.client.img2img()?;
        let resp = img2img.send(config).await?;
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
