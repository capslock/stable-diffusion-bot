use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub struct Img2ImgRequest {
    pub init_images: Option<Vec<String>>,
    pub resize_mode: Option<u32>,
    pub denoising_strength: Option<f64>,
    pub image_cfg_scale: Option<u32>,
    pub mask: Option<String>,
    pub mask_blur: Option<u32>,
    pub inpainting_fill: Option<u32>,
    pub inpaint_full_res: Option<bool>,
    pub inpaint_full_res_padding: Option<u32>,
    pub inpainting_mask_invert: Option<u32>,
    pub initial_noise_multiplier: Option<u32>,
    pub prompt: Option<String>,
    pub styles: Option<Vec<String>>,
    pub seed: Option<i32>,
    pub subseed: Option<i32>,
    pub subseed_strength: Option<u32>,
    pub seed_resize_from_h: Option<i32>,
    pub seed_resize_from_w: Option<i32>,
    pub sampler_name: Option<String>,
    pub batch_size: Option<u32>,
    pub n_iter: Option<u32>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub restore_faces: Option<bool>,
    pub tiling: Option<bool>,
    pub do_not_save_samples: Option<bool>,
    pub do_not_save_grid: Option<bool>,
    pub negative_prompt: Option<String>,
    pub eta: Option<u32>,
    pub s_churn: Option<u32>,
    pub s_tmax: Option<u32>,
    pub s_tmin: Option<u32>,
    pub s_noise: Option<u32>,
    pub override_settings: Option<HashMap<String, serde_json::Value>>,
    pub override_settings_restore_afterwards: Option<bool>,
    pub script_args: Option<Vec<serde_json::Value>>,
    pub sampler_index: Option<String>,
    pub include_init_images: Option<bool>,
    pub script_name: Option<String>,
    pub send_images: Option<bool>,
    pub save_images: Option<bool>,
    pub alwayson_scripts: Option<HashMap<String, serde_json::Value>>,
}

impl Img2ImgRequest {
    pub fn with_prompt(&mut self, prompt: String) -> &mut Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn with_styles(&mut self, styles: Vec<String>) -> &mut Self {
        self.styles = Some(styles);
        self
    }

    pub fn with_style(&mut self, style: String) -> &mut Self {
        if let Some(ref mut styles) = self.styles {
            styles.push(style);
            self
        } else {
            self.with_styles(vec![style])
        }
    }

    pub fn with_denoising_strength(&mut self, denoising_strength: f64) -> &mut Self {
        self.denoising_strength = Some(denoising_strength);
        self
    }

    pub fn with_seed(&mut self, seed: i32) -> &mut Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_subseed(&mut self, subseed: i32) -> &mut Self {
        self.subseed = Some(subseed);
        self
    }

    pub fn with_subseed_strength(&mut self, subseed_strength: u32) -> &mut Self {
        self.subseed_strength = Some(subseed_strength);
        self
    }

    pub fn with_sampler_name(&mut self, sampler_name: String) -> &mut Self {
        self.sampler_name = Some(sampler_name);
        self
    }

    pub fn with_batch_size(&mut self, batch_size: u32) -> &mut Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn with_n_iter(&mut self, n_iter: u32) -> &mut Self {
        self.n_iter = Some(n_iter);
        self
    }

    pub fn with_steps(&mut self, steps: u32) -> &mut Self {
        self.steps = Some(steps);
        self
    }

    pub fn with_cfg_scale(&mut self, cfg_scale: u32) -> &mut Self {
        self.cfg_scale = Some(cfg_scale);
        self
    }

    pub fn with_width(&mut self, width: u32) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(&mut self, height: u32) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn with_restore_faces(&mut self, restore_faces: bool) -> &mut Self {
        self.restore_faces = Some(restore_faces);
        self
    }

    pub fn with_tiling(&mut self, tiling: bool) -> &mut Self {
        self.tiling = Some(tiling);
        self
    }

    pub fn with_negative_prompt(&mut self, negative_prompt: String) -> &mut Self {
        self.negative_prompt = Some(negative_prompt);
        self
    }
}
