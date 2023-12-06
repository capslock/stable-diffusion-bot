use anyhow::Context as _;
use comfyui_api::{comfy::getter::*, models::AsAny};
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use stable_diffusion_api::{Img2ImgRequest, Txt2ImgRequest};

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
        self.prompt.seed().ok().copied()
    }

    fn set_seed(&mut self, seed: i64) {
        self.prompt.seed_mut().map(|s| *s = seed).ok();
    }

    fn steps(&self) -> Option<u32> {
        self.prompt.steps().ok().copied()
    }

    fn set_steps(&mut self, steps: u32) {
        self.prompt.steps_mut().map(|s| *s = steps).ok();
    }

    fn count(&self) -> Option<u32> {
        Some(self.count)
    }

    fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    fn cfg(&self) -> Option<f32> {
        self.prompt.cfg().ok().copied()
    }

    fn set_cfg(&mut self, cfg: f32) {
        self.prompt.cfg_mut().map(|s| *s = cfg).ok();
    }

    fn width(&self) -> Option<u32> {
        self.prompt.width().ok().copied()
    }

    fn set_width(&mut self, width: u32) {
        self.prompt.width_mut().map(|s| *s = width).ok();
    }

    fn height(&self) -> Option<u32> {
        self.prompt.height().ok().copied()
    }

    fn set_height(&mut self, height: u32) {
        self.prompt.height_mut().map(|s| *s = height).ok();
    }

    fn prompt(&self) -> Option<String> {
        self.prompt.prompt().ok().cloned()
    }

    fn set_prompt(&mut self, prompt: String) {
        self.prompt.prompt_mut().map(|s| *s = prompt).ok();
    }

    fn negative_prompt(&self) -> Option<String> {
        self.prompt.negative_prompt().ok().cloned()
    }

    fn set_negative_prompt(&mut self, negative_prompt: String) {
        self.prompt
            .negative_prompt_mut()
            .map(|s| *s = negative_prompt)
            .ok();
    }

    fn denoising(&self) -> Option<f32> {
        self.prompt.denoise().ok().copied()
    }

    fn set_denoising(&mut self, denoising: f32) {
        self.prompt.denoise_mut().map(|s| *s = denoising).ok();
    }

    fn sampler(&self) -> Option<String> {
        self.prompt.sampler_name().ok().cloned()
    }

    fn set_sampler(&mut self, sampler: String) {
        self.prompt.sampler_name_mut().map(|s| *s = sampler).ok();
    }

    fn batch_size(&self) -> Option<u32> {
        self.prompt.batch_size().ok().copied()
    }

    fn set_batch_size(&mut self, batch_size: u32) {
        self.prompt.batch_size_mut().map(|s| *s = batch_size).ok();
    }

    fn image(&self) -> Option<Vec<u8>> {
        self.image.clone()
    }

    fn set_image(&mut self, image: Option<Vec<u8>>) {
        self.image = image;
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
