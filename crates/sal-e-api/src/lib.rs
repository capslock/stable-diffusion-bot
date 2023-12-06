use anyhow::Context;
use async_trait::async_trait;
use comfyui_api::{comfy::LoadImageExt, models::AsAny};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, ImgInfo, Txt2ImgRequest};

#[derive(Debug, Clone)]
pub struct Response {
    pub images: Vec<Vec<u8>>,
    pub params: Box<dyn ImageParams>,
    pub gen_params: Box<dyn GenParams>,
}

dyn_clone::clone_trait_object!(ImageParams);

pub trait ImageParams: std::fmt::Debug + AsAny + Send + Sync + DynClone {
    fn seed(&self) -> Option<i64>;
    fn steps(&self) -> Option<u32>;
    fn cfg(&self) -> Option<f32>;
    fn width(&self) -> Option<u32>;
    fn height(&self) -> Option<u32>;
    fn prompt(&self) -> Option<String>;
    fn negative_prompt(&self) -> Option<String>;
    fn denoising(&self) -> Option<f32>;
    fn model(&self) -> Option<String>;
    fn sampler(&self) -> Option<String>;
}

impl ImageParams for comfyui_api::models::Prompt {
    fn seed(&self) -> Option<i64> {
        comfyui_api::comfy::SeedExt::seed(self).ok().copied()
    }

    fn steps(&self) -> Option<u32> {
        comfyui_api::comfy::StepsExt::steps(self).ok().copied()
    }

    fn cfg(&self) -> Option<f32> {
        comfyui_api::comfy::CfgExt::cfg(self).ok().copied()
    }

    fn width(&self) -> Option<u32> {
        comfyui_api::comfy::WidthExt::width(self).ok().copied()
    }

    fn height(&self) -> Option<u32> {
        comfyui_api::comfy::HeightExt::height(self).ok().copied()
    }

    fn prompt(&self) -> Option<String> {
        comfyui_api::comfy::PromptExt::prompt(self).ok().cloned()
    }

    fn negative_prompt(&self) -> Option<String> {
        comfyui_api::comfy::NegativePromptExt::negative_prompt(self)
            .ok()
            .cloned()
    }

    fn denoising(&self) -> Option<f32> {
        comfyui_api::comfy::DenoiseExt::denoise(self).ok().copied()
    }

    fn model(&self) -> Option<String> {
        comfyui_api::comfy::ModelExt::ckpt_name(self).ok().cloned()
    }

    fn sampler(&self) -> Option<String> {
        comfyui_api::comfy::SamplerExt::sampler_name(self)
            .ok()
            .cloned()
    }
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

    fn sampler(&self) -> Option<String>;
    fn set_sampler(&mut self, sampler: String);

    fn batch_size(&self) -> Option<u32>;
    fn set_batch_size(&mut self, batch_size: u32);

    fn image(&self) -> Option<Vec<u8>>;
    fn set_image(&mut self, image: Option<Vec<u8>>);
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComfyParams {
    pub prompt: comfyui_api::models::Prompt,
    pub count: u32,
    pub image: Option<Vec<u8>>,
}

#[typetag::serde]
impl GenParams for ComfyParams {
    fn seed(&self) -> Option<i64> {
        comfyui_api::comfy::SeedExt::seed(&self.prompt)
            .ok()
            .copied()
    }

    fn set_seed(&mut self, seed: i64) {
        comfyui_api::comfy::SeedExt::seed_mut(&mut self.prompt)
            .map(|s| *s = seed)
            .ok();
    }

    fn steps(&self) -> Option<u32> {
        comfyui_api::comfy::StepsExt::steps(&self.prompt)
            .ok()
            .copied()
    }

    fn set_steps(&mut self, steps: u32) {
        comfyui_api::comfy::StepsExt::steps_mut(&mut self.prompt)
            .map(|s| *s = steps)
            .ok();
    }

    fn count(&self) -> Option<u32> {
        Some(self.count)
    }

    fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    fn cfg(&self) -> Option<f32> {
        comfyui_api::comfy::CfgExt::cfg(&self.prompt).ok().copied()
    }

    fn set_cfg(&mut self, cfg: f32) {
        comfyui_api::comfy::CfgExt::cfg_mut(&mut self.prompt)
            .map(|s| *s = cfg)
            .ok();
    }

    fn width(&self) -> Option<u32> {
        comfyui_api::comfy::WidthExt::width(&self.prompt)
            .ok()
            .copied()
    }

    fn set_width(&mut self, width: u32) {
        comfyui_api::comfy::WidthExt::width_mut(&mut self.prompt)
            .map(|s| *s = width)
            .ok();
    }

    fn height(&self) -> Option<u32> {
        comfyui_api::comfy::HeightExt::height(&self.prompt)
            .ok()
            .copied()
    }

    fn set_height(&mut self, height: u32) {
        comfyui_api::comfy::HeightExt::height_mut(&mut self.prompt)
            .map(|s| *s = height)
            .ok();
    }

    fn prompt(&self) -> Option<String> {
        comfyui_api::comfy::PromptExt::prompt(&self.prompt)
            .ok()
            .cloned()
    }

    fn set_prompt(&mut self, prompt: String) {
        comfyui_api::comfy::PromptExt::prompt_mut(&mut self.prompt)
            .map(|s| *s = prompt)
            .ok();
    }

    fn negative_prompt(&self) -> Option<String> {
        comfyui_api::comfy::NegativePromptExt::negative_prompt(&self.prompt)
            .ok()
            .cloned()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        comfyui_api::comfy::NegativePromptExt::negative_prompt_mut(&mut self.prompt)
            .map(|s| *s = negative_prompt)
            .ok();
    }

    fn denoising(&self) -> Option<f32> {
        comfyui_api::comfy::DenoiseExt::denoise(&self.prompt)
            .ok()
            .copied()
    }

    fn set_denoising(&mut self, denoising: f32) {
        comfyui_api::comfy::DenoiseExt::denoise_mut(&mut self.prompt)
            .map(|s| *s = denoising)
            .ok();
    }

    fn sampler(&self) -> Option<String> {
        comfyui_api::comfy::SamplerExt::sampler_name(&self.prompt)
            .ok()
            .cloned()
    }

    fn set_sampler(&mut self, sampler: String) {
        comfyui_api::comfy::SamplerExt::sampler_name_mut(&mut self.prompt)
            .map(|s| *s = sampler)
            .ok();
    }

    fn batch_size(&self) -> Option<u32> {
        comfyui_api::comfy::BatchSizeExt::batch_size(&self.prompt)
            .ok()
            .copied()
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        comfyui_api::comfy::BatchSizeExt::batch_size_mut(&mut self.prompt)
            .map(|s| *s = batch_size)
            .ok();
    }

    fn image(&self) -> Option<Vec<u8>> {
        self.image.clone()
    }

    fn set_image(&mut self, image: Option<Vec<u8>>) {
        self.image = image;
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
            params: ComfyParams {
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
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Response>;

    fn gen_params(&self) -> Box<dyn GenParams>;
}

dyn_clone::clone_trait_object!(Img2ImgApi);

/// Struct representing a connection to a Stable Diffusion API.
#[async_trait]
pub trait Img2ImgApi: std::fmt::Debug + DynClone + Send + Sync + AsAny {
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Response>;

    fn gen_params(&self) -> Box<dyn GenParams>;
}

#[async_trait]
impl Txt2ImgApi for ComfyPromptApi {
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Response> {
        let prompt = config.as_any().downcast_ref().unwrap_or(&self.params);
        let images = self.client.execute_prompt(&prompt.prompt).await?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(prompt.prompt.clone()),
            gen_params: Box::new(prompt.clone()),
        })
    }

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.params.clone())
    }
}

#[async_trait]
impl Img2ImgApi for ComfyPromptApi {
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Response> {
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

        let images = self.client.execute_prompt(&prompt).await?;
        Ok(Response {
            images: images.into_iter().map(|image| image.image).collect(),
            params: Box::new(base_prompt.prompt.clone()),
            gen_params: Box::new(base_prompt.clone()),
        })
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

    fn sampler(&self) -> Option<String> {
        self.sampler_index.clone()
    }

    fn set_sampler(&mut self, sampler: String) {
        self.sampler_index = Some(sampler);
    }

    fn batch_size(&self) -> Option<u32> {
        self.batch_size
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.batch_size = Some(batch_size);
    }

    fn image(&self) -> Option<Vec<u8>> {
        None
    }

    fn set_image(&mut self, _image: Option<Vec<u8>>) {}
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

    fn sampler(&self) -> Option<String> {
        self.sampler_index.clone()
    }

    fn set_sampler(&mut self, sampler: String) {
        self.sampler_index = Some(sampler);
    }

    fn batch_size(&self) -> Option<u32> {
        self.batch_size
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.batch_size = Some(batch_size);
    }

    fn image(&self) -> Option<Vec<u8>> {
        if let Some(ref images) = self.init_images {
            use base64::{engine::general_purpose, Engine as _};
            images
                .iter()
                .map(|img| {
                    general_purpose::STANDARD
                        .decode(img)
                        .context("failed to decode image")
                })
                .collect::<anyhow::Result<Vec<_>>>()
                .ok()
                .and_then(|mut images| images.pop())
        } else {
            None
        }
    }

    fn set_image(&mut self, image: Option<Vec<u8>>) {
        if let Some(image) = image {
            self.with_image(image);
        } else {
            _ = self.init_images.take()
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
    async fn txt2img(&self, config: &dyn GenParams) -> anyhow::Result<Response> {
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

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.txt2img_defaults.clone())
    }
}

#[async_trait]
impl Img2ImgApi for StableDiffusionWebUiApi {
    async fn img2img(&self, config: &dyn GenParams) -> anyhow::Result<Response> {
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

    fn gen_params(&self) -> Box<dyn GenParams> {
        Box::new(self.img2img_defaults.clone())
    }
}

impl ImageParams for ImgInfo {
    fn seed(&self) -> Option<i64> {
        self.seed
    }

    fn steps(&self) -> Option<u32> {
        self.steps
    }

    fn cfg(&self) -> Option<f32> {
        self.cfg_scale.map(|c| c as f32)
    }

    fn width(&self) -> Option<u32> {
        self.width.map(|w| w as u32)
    }

    fn height(&self) -> Option<u32> {
        self.height.map(|h| h as u32)
    }

    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt.clone()
    }

    fn denoising(&self) -> Option<f32> {
        self.denoising_strength.map(|d| d as f32)
    }

    fn model(&self) -> Option<String> {
        self.sd_model_name.clone()
    }

    fn sampler(&self) -> Option<String> {
        self.sampler_name.clone()
    }
}
