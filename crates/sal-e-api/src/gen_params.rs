use anyhow::Context as _;
use comfyui_api::{
    comfy::getter::*,
    models::{AsAny, Prompt},
};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

dyn_clone::clone_trait_object!(GenParams);

/// Trait representing an interface to image generation parameters.
#[typetag::serde]
pub trait GenParams: std::fmt::Debug + AsAny + Send + Sync + DynClone {
    /// Gets the seed.
    fn seed(&self) -> Option<i64>;
    /// Sets the seed.
    fn set_seed(&mut self, seed: i64);

    /// Gets the number of steps.
    fn steps(&self) -> Option<u32>;
    /// Sets the number of steps.
    fn set_steps(&mut self, steps: u32);

    /// Gets the number of images to generate.
    fn count(&self) -> Option<u32>;
    /// Sets the number of images to generate.
    fn set_count(&mut self, count: u32);

    /// Gets the CFG scale.
    fn cfg(&self) -> Option<f32>;
    /// Sets the CFG scale.
    fn set_cfg(&mut self, cfg: f32);

    /// Gets the image width.
    fn width(&self) -> Option<u32>;
    /// Sets the image width.
    fn set_width(&mut self, width: u32);

    /// Gets the image height.
    fn height(&self) -> Option<u32>;
    /// Sets the image height.
    fn set_height(&mut self, height: u32);

    /// Gets the prompt.
    fn prompt(&self) -> Option<String>;
    /// Sets the prompt.
    fn set_prompt(&mut self, prompt: String);

    /// Gets the negative prompt.
    fn negative_prompt(&self) -> Option<String>;
    /// Sets the negative prompt.
    fn set_negative_prompt(&mut self, negative_prompt: String);

    /// Gets the denoising strength.
    fn denoising(&self) -> Option<f32>;
    /// Sets the denoising strength.
    fn set_denoising(&mut self, denoising: f32);

    /// Gets the sampler.
    fn sampler(&self) -> Option<String>;
    /// Sets the sampler.
    fn set_sampler(&mut self, sampler: String);

    /// Gets the batch size.
    fn batch_size(&self) -> Option<u32>;
    /// Sets the batch size.
    fn set_batch_size(&mut self, batch_size: u32);

    /// Gets the image.
    fn image(&self) -> Option<Vec<u8>>;
    /// Sets the image.
    fn set_image(&mut self, image: Option<Vec<u8>>);
}

/// A struct representing the parameters for ComfyUI image generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComfyParams {
    /// The ComfyUI prompt to use for generation.
    #[serde(skip)]
    pub prompt: Option<comfyui_api::models::Prompt>,
    /// The random seed to use for generation.
    pub seed: Option<i64>,
    /// The number of steps to take for generation.
    pub steps: Option<u32>,
    /// The number of images to generate.
    pub count: u32,
    /// The CFG scale to use for generation.
    pub cfg: Option<f32>,
    /// The image width to use for generation.
    pub width: Option<u32>,
    /// The image height to use for generation.
    pub height: Option<u32>,
    /// The prompt text to use for generation.
    pub prompt_text: Option<String>,
    /// The negative prompt text to use for generation.
    pub negative_prompt_text: Option<String>,
    /// The denoising strength to use for generation.
    pub denoising: Option<f32>,
    /// The sampler to use for generation.
    pub sampler: Option<String>,
    /// The batch size to use for generation.
    pub batch_size: Option<u32>,
    /// The image to use for generation.
    pub image: Option<Vec<u8>>,
}

impl ComfyParams {
    /// Applies the parameters to the provided prompt.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to apply the parameters to.
    ///
    /// # Returns
    ///
    /// The prompt with the parameters applied.
    pub fn apply_to(&self, prompt: &Prompt) -> Prompt {
        let mut prompt = prompt.clone();

        if let Some(seed) = self.seed {
            _ = prompt.seed_mut().map(|s| *s = seed);
        }

        if let Some(steps) = self.steps {
            _ = prompt.steps_mut().map(|s| *s = steps);
        }

        if let Some(cfg) = self.cfg {
            _ = prompt.cfg_mut().map(|c| *c = cfg);
        }

        if let Some(width) = self.width {
            _ = prompt.width_mut().map(|w| *w = width);
        }

        if let Some(height) = self.height {
            _ = prompt.height_mut().map(|h| *h = height);
        }

        if let Some(prompt_text) = &self.prompt_text {
            _ = prompt.prompt_mut().map(|p| *p = prompt_text.clone());
        }

        if let Some(negative_prompt_text) = &self.negative_prompt_text {
            _ = prompt
                .negative_prompt_mut()
                .map(|p| *p = negative_prompt_text.clone());
        }

        if let Some(denoising) = self.denoising {
            _ = prompt.denoise_mut().map(|d| *d = denoising);
        }

        if let Some(sampler) = &self.sampler {
            _ = prompt.sampler_name_mut().map(|s| *s = sampler.clone());
        }

        if let Some(batch_size) = self.batch_size {
            _ = prompt.batch_size_mut().map(|b| *b = batch_size);
        }

        prompt
    }

    /// Applies the parameters to the current prompt.
    ///
    /// # Returns
    ///
    /// The prompt with the parameters applied.
    pub fn apply(&self) -> Option<Prompt> {
        self.prompt.as_ref().map(|prompt| self.apply_to(prompt))
    }
}

impl From<&dyn GenParams> for ComfyParams {
    fn from(params: &dyn GenParams) -> Self {
        Self {
            seed: params.seed(),
            steps: params.steps(),
            count: params.count().unwrap_or(1),
            cfg: params.cfg(),
            width: params.width(),
            height: params.height(),
            prompt_text: params.prompt(),
            negative_prompt_text: params.negative_prompt(),
            denoising: params.denoising(),
            sampler: params.sampler(),
            batch_size: params.batch_size(),
            image: params.image(),
            ..Default::default()
        }
    }
}

#[typetag::serde]
impl GenParams for ComfyParams {
    fn seed(&self) -> Option<i64> {
        self.seed
            .or_else(|| self.prompt.as_ref()?.seed().ok().copied())
    }

    fn set_seed(&mut self, seed: i64) {
        self.seed = Some(seed);
    }

    fn steps(&self) -> Option<u32> {
        self.steps
            .or_else(|| self.prompt.as_ref()?.steps().ok().copied())
    }

    fn set_steps(&mut self, steps: u32) {
        self.steps = Some(steps);
    }

    fn count(&self) -> Option<u32> {
        Some(self.count)
    }

    fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    fn cfg(&self) -> Option<f32> {
        self.cfg
            .or_else(|| self.prompt.as_ref()?.cfg().ok().copied())
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.cfg = Some(cfg);
    }

    fn width(&self) -> Option<u32> {
        self.width
            .or_else(|| self.prompt.as_ref()?.width().ok().copied())
    }

    fn set_width(&mut self, width: u32) {
        self.width = Some(width);
    }

    fn height(&self) -> Option<u32> {
        self.height
            .or_else(|| self.prompt.as_ref()?.height().ok().copied())
    }

    fn set_height(&mut self, height: u32) {
        self.height = Some(height);
    }

    fn prompt(&self) -> Option<String> {
        self.prompt_text
            .clone()
            .or_else(|| self.prompt.as_ref()?.prompt().ok().cloned())
    }

    fn set_prompt(&mut self, prompt: String) {
        self.prompt_text = Some(prompt);
    }

    fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt_text
            .clone()
            .or_else(|| self.prompt.as_ref()?.negative_prompt().ok().cloned())
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.negative_prompt_text = Some(negative_prompt);
    }

    fn denoising(&self) -> Option<f32> {
        self.denoising
            .or_else(|| self.prompt.as_ref()?.denoise().ok().copied())
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.denoising = Some(denoising);
    }

    fn sampler(&self) -> Option<String> {
        self.sampler
            .clone()
            .or_else(|| self.prompt.as_ref()?.sampler_name().ok().cloned())
    }

    fn set_sampler(&mut self, sampler: String) {
        self.sampler = Some(sampler);
    }

    fn batch_size(&self) -> Option<u32> {
        self.batch_size
            .or_else(|| self.prompt.as_ref()?.batch_size().ok().copied())
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.batch_size = Some(batch_size);
    }

    fn image(&self) -> Option<Vec<u8>> {
        self.image.clone()
    }

    fn set_image(&mut self, image: Option<Vec<u8>>) {
        self.image = image;
    }
}

/// A struct representing the parameters for image generation in the Stable Diffusion WebUI API.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Txt2ImgParams {
    /// The parameters provided by the user.
    pub user_params: Txt2ImgRequest,
    /// The default parameters.
    #[serde(skip)]
    pub defaults: Option<Txt2ImgRequest>,
}

impl From<&dyn GenParams> for Txt2ImgParams {
    fn from(params: &dyn GenParams) -> Self {
        Self {
            user_params: Txt2ImgRequest {
                seed: params.seed(),
                steps: params.steps(),
                n_iter: params.count(),
                cfg_scale: params.cfg().map(|c| c as f64),
                width: params.width(),
                height: params.height(),
                prompt: params.prompt(),
                negative_prompt: params.negative_prompt(),
                denoising_strength: params.denoising().map(|d| d as f64),
                sampler_index: params.sampler(),
                batch_size: params.batch_size(),
                ..Default::default()
            },
            defaults: None,
        }
    }
}

#[typetag::serde]
impl GenParams for Txt2ImgParams {
    fn seed(&self) -> Option<i64> {
        self.user_params
            .seed
            .or_else(|| self.defaults.as_ref()?.seed)
    }

    fn set_seed(&mut self, seed: i64) {
        self.user_params.seed = Some(seed);
    }

    fn steps(&self) -> Option<u32> {
        self.user_params
            .steps
            .or_else(|| self.defaults.as_ref()?.steps)
    }

    fn set_steps(&mut self, steps: u32) {
        self.user_params.steps = Some(steps);
    }

    fn count(&self) -> Option<u32> {
        self.user_params
            .n_iter
            .or_else(|| self.defaults.as_ref()?.n_iter)
    }

    fn set_count(&mut self, count: u32) {
        self.user_params.n_iter = Some(count);
    }

    fn cfg(&self) -> Option<f32> {
        self.user_params
            .cfg_scale
            .map(|c| c as f32)
            .or_else(|| self.defaults.as_ref()?.cfg_scale.map(|c| c as f32))
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.user_params.cfg_scale = Some(cfg as f64);
    }

    fn width(&self) -> Option<u32> {
        self.user_params
            .width
            .or_else(|| self.defaults.as_ref()?.width)
    }

    fn set_width(&mut self, width: u32) {
        self.user_params.width = Some(width);
    }

    fn height(&self) -> Option<u32> {
        self.user_params
            .height
            .or_else(|| self.defaults.as_ref()?.height)
    }

    fn set_height(&mut self, height: u32) {
        self.user_params.height = Some(height);
    }

    fn prompt(&self) -> Option<String> {
        self.user_params
            .prompt
            .clone()
            .or_else(|| self.defaults.as_ref()?.prompt.clone())
    }

    fn set_prompt(&mut self, prompt: String) {
        self.user_params.prompt = Some(prompt);
    }

    fn negative_prompt(&self) -> Option<String> {
        self.user_params
            .negative_prompt
            .clone()
            .or_else(|| self.defaults.as_ref()?.negative_prompt.clone())
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.user_params.negative_prompt = Some(negative_prompt);
    }

    fn denoising(&self) -> Option<f32> {
        self.user_params
            .denoising_strength
            .map(|d| d as f32)
            .or_else(|| self.defaults.as_ref()?.denoising_strength.map(|d| d as f32))
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.user_params.denoising_strength = Some(denoising as f64);
    }

    fn sampler(&self) -> Option<String> {
        self.user_params
            .sampler_index
            .clone()
            .or_else(|| self.defaults.as_ref()?.sampler_index.clone())
    }

    fn set_sampler(&mut self, sampler: String) {
        self.user_params.sampler_index = Some(sampler);
    }

    fn batch_size(&self) -> Option<u32> {
        self.user_params
            .batch_size
            .or_else(|| self.defaults.as_ref()?.batch_size)
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.user_params.batch_size = Some(batch_size);
    }

    fn image(&self) -> Option<Vec<u8>> {
        None
    }

    fn set_image(&mut self, _image: Option<Vec<u8>>) {}
}

/// A struct representing the parameters for image generation in the Stable Diffusion WebUI API.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Img2ImgParams {
    /// The parameters provided by the user.
    pub user_params: Img2ImgRequest,
    /// The default parameters.
    #[serde(skip)]
    pub defaults: Option<Img2ImgRequest>,
}

impl From<&dyn GenParams> for Img2ImgParams {
    fn from(params: &dyn GenParams) -> Self {
        Self {
            user_params: Img2ImgRequest {
                seed: params.seed(),
                steps: params.steps(),
                n_iter: params.count(),
                cfg_scale: params.cfg().map(|c| c as f64),
                width: params.width(),
                height: params.height(),
                prompt: params.prompt(),
                negative_prompt: params.negative_prompt(),
                denoising_strength: params.denoising().map(|d| d as f64),
                sampler_index: params.sampler(),
                batch_size: params.batch_size(),
                ..Default::default()
            },
            defaults: None,
        }
    }
}

#[typetag::serde]
impl GenParams for Img2ImgParams {
    fn seed(&self) -> Option<i64> {
        self.user_params
            .seed
            .or_else(|| self.defaults.as_ref()?.seed)
    }

    fn set_seed(&mut self, seed: i64) {
        self.user_params.seed = Some(seed);
    }

    fn steps(&self) -> Option<u32> {
        self.user_params
            .steps
            .or_else(|| self.defaults.as_ref()?.steps)
    }

    fn set_steps(&mut self, steps: u32) {
        self.user_params.steps = Some(steps);
    }

    fn count(&self) -> Option<u32> {
        self.user_params
            .n_iter
            .or_else(|| self.defaults.as_ref()?.n_iter)
    }

    fn set_count(&mut self, count: u32) {
        self.user_params.n_iter = Some(count);
    }

    fn cfg(&self) -> Option<f32> {
        self.user_params
            .cfg_scale
            .map(|c| c as f32)
            .or_else(|| self.defaults.as_ref()?.cfg_scale.map(|c| c as f32))
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.user_params.cfg_scale = Some(cfg as f64);
    }

    fn width(&self) -> Option<u32> {
        self.user_params
            .width
            .or_else(|| self.defaults.as_ref()?.width)
    }

    fn set_width(&mut self, width: u32) {
        self.user_params.width = Some(width);
    }

    fn height(&self) -> Option<u32> {
        self.user_params
            .height
            .or_else(|| self.defaults.as_ref()?.height)
    }

    fn set_height(&mut self, height: u32) {
        self.user_params.height = Some(height);
    }

    fn prompt(&self) -> Option<String> {
        self.user_params
            .prompt
            .clone()
            .or_else(|| self.defaults.as_ref()?.prompt.clone())
    }

    fn set_prompt(&mut self, prompt: String) {
        self.user_params.prompt = Some(prompt);
    }

    fn negative_prompt(&self) -> Option<String> {
        self.user_params
            .negative_prompt
            .clone()
            .or_else(|| self.defaults.as_ref()?.negative_prompt.clone())
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.user_params.negative_prompt = Some(negative_prompt);
    }

    fn denoising(&self) -> Option<f32> {
        self.user_params
            .denoising_strength
            .map(|d| d as f32)
            .or_else(|| self.defaults.as_ref()?.denoising_strength.map(|d| d as f32))
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.user_params.denoising_strength = Some(denoising as f64);
    }

    fn sampler(&self) -> Option<String> {
        self.user_params
            .sampler_index
            .clone()
            .or_else(|| self.defaults.as_ref()?.sampler_index.clone())
    }

    fn set_sampler(&mut self, sampler: String) {
        self.user_params.sampler_index = Some(sampler);
    }

    fn batch_size(&self) -> Option<u32> {
        self.user_params
            .batch_size
            .or_else(|| self.defaults.as_ref()?.batch_size)
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.user_params.batch_size = Some(batch_size);
    }

    fn image(&self) -> Option<Vec<u8>> {
        if let Some(ref images) = self.user_params.init_images {
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
            self.user_params.with_image(image);
        } else {
            _ = self.user_params.init_images.take()
        }
    }
}
