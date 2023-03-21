use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Txt2ImgResponse {
    pub images: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub info: String,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub struct Txt2ImgRequest {
    pub enable_hr: Option<bool>,
    pub denoising_strength: Option<u32>,
    pub firstphase_width: Option<u32>,
    pub firstphase_height: Option<u32>,
    pub hr_scale: Option<u32>,
    pub hr_upscaler: Option<String>,
    pub hr_second_pass_steps: Option<u32>,
    pub hr_resize_x: Option<u32>,
    pub hr_resize_y: Option<u32>,
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
    pub script_name: Option<String>,
    pub send_images: Option<bool>,
    pub save_images: Option<bool>,
    pub alwayson_scripts: Option<HashMap<String, serde_json::Value>>,
}

impl Txt2ImgRequest {
    pub fn with_prompt(self, prompt: String) -> Self {
        Self {
            prompt: Some(prompt),
            ..self
        }
    }

    pub fn with_styles(self, styles: Vec<String>) -> Self {
        Self {
            styles: Some(styles),
            ..self
        }
    }

    pub fn with_style(mut self, style: String) -> Self {
        if let Some(ref mut styles) = &mut self.styles {
            styles.push(style);
            self
        } else {
            self.with_styles(vec![style])
        }
    }

    pub fn with_seed(self, seed: i32) -> Self {
        Self {
            seed: Some(seed),
            ..self
        }
    }

    pub fn with_subseed(self, subseed: i32) -> Self {
        Self {
            subseed: Some(subseed),
            ..self
        }
    }

    pub fn with_subseed_strength(self, subseed_strength: u32) -> Self {
        Self {
            subseed_strength: Some(subseed_strength),
            ..self
        }
    }

    pub fn with_sampler_name(self, sampler_name: String) -> Self {
        Self {
            sampler_name: Some(sampler_name),
            ..self
        }
    }

    pub fn with_batch_size(self, batch_size: u32) -> Self {
        Self {
            batch_size: Some(batch_size),
            ..self
        }
    }

    pub fn with_n_iter(self, n_iter: u32) -> Self {
        Self {
            n_iter: Some(n_iter),
            ..self
        }
    }

    pub fn with_steps(self, steps: u32) -> Self {
        Self {
            steps: Some(steps),
            ..self
        }
    }

    pub fn with_cfg_scale(self, cfg_scale: u32) -> Self {
        Self {
            cfg_scale: Some(cfg_scale),
            ..self
        }
    }

    pub fn with_width(self, width: u32) -> Self {
        Self {
            width: Some(width),
            ..self
        }
    }

    pub fn with_height(self, height: u32) -> Self {
        Self {
            height: Some(height),
            ..self
        }
    }

    pub fn with_restore_faces(self, restore_faces: bool) -> Self {
        Self {
            restore_faces: Some(restore_faces),
            ..self
        }
    }

    pub fn with_tiling(self, tiling: bool) -> Self {
        Self {
            tiling: Some(tiling),
            ..self
        }
    }

    pub fn with_negative_prompt(self, negative_prompt: String) -> Self {
        Self {
            negative_prompt: Some(negative_prompt),
            ..self
        }
    }
}

pub struct Txt2Img {
    client: reqwest::Client,
    endpoint: String,
}

impl Txt2Img {
    pub fn new(client: reqwest::Client, endpoint: String) -> Self {
        Self { client, endpoint }
    }

    pub async fn send(&self, request: &Txt2ImgRequest) -> anyhow::Result<Txt2ImgResponse> {
        let res = self
            .client
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .context("failed to send request")?;
        res.json().await.context("failed to parse json")
    }
}
