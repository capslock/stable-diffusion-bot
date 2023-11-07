use std::collections::HashMap;

use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::ImgResponse;

/// Struct representing a text to image request.
#[skip_serializing_none]
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Txt2ImgRequest {
    /// Whether to enable high resolution mode.
    pub enable_hr: Option<bool>,
    /// Strength of denoising applied to the image.
    pub denoising_strength: Option<f64>,
    /// Width of the image in the first phase.
    pub firstphase_width: Option<u32>,
    /// Height of the image in the first phase.
    pub firstphase_height: Option<u32>,
    /// Scale factor for high resolution mode.
    pub hr_scale: Option<f64>,
    /// Upscaler used in high resolution mode.
    pub hr_upscaler: Option<String>,
    /// Number of steps in the second pass of high resolution mode.
    pub hr_second_pass_steps: Option<u32>,
    /// Width of the image after resizing in high resolution mode.
    pub hr_resize_x: Option<u32>,
    /// Height of the image after resizing in high resolution mode.
    pub hr_resize_y: Option<u32>,
    /// Text prompt for generating the image.
    pub prompt: Option<String>,
    /// List of style prompts for generating the image.
    pub styles: Option<Vec<String>>,
    /// Seed for generating the image.
    pub seed: Option<i64>,
    /// Subseed for generating the image.
    pub subseed: Option<i64>,
    /// Strength of subseed.
    pub subseed_strength: Option<u32>,
    /// Height of the seed image.
    pub seed_resize_from_h: Option<i32>,
    /// Width of the seed image.
    pub seed_resize_from_w: Option<i32>,
    /// Name of the sampler.
    pub sampler_name: Option<String>,
    /// Batch size used in generating images.
    pub batch_size: Option<u32>,
    /// Number of images to generate per batch.
    pub n_iter: Option<u32>,
    /// Number of steps.
    pub steps: Option<u32>,
    /// CFG scale factor.
    pub cfg_scale: Option<f64>,
    /// Width of the generated image.
    pub width: Option<u32>,
    /// Height of the generated image.
    pub height: Option<u32>,
    /// Whether to restore faces in the generated image.
    pub restore_faces: Option<bool>,
    /// Whether to use tiling mode in the generated image.
    pub tiling: Option<bool>,
    /// Whether to save samples when generating multiple images.
    pub do_not_save_samples: Option<bool>,
    /// Whether to save the grid when generating multiple images.
    pub do_not_save_grid: Option<bool>,
    /// Negative text prompt.
    pub negative_prompt: Option<String>,
    /// Eta value.
    pub eta: Option<u32>,
    /// Churn value.
    pub s_churn: Option<f64>,
    /// Maximum temperature value.
    pub s_tmax: Option<f64>,
    /// Minimum temperature value.
    pub s_tmin: Option<f64>,
    /// Noise value.
    pub s_noise: Option<f64>,
    /// Settings to override when generating the image.
    pub override_settings: Option<HashMap<String, serde_json::Value>>,
    /// Whether to restore the settings after generating the image.
    pub override_settings_restore_afterwards: Option<bool>,
    /// Arguments for the script.
    pub script_args: Option<Vec<serde_json::Value>>,
    /// Index of the sampler.
    pub sampler_index: Option<String>,
    /// Name of the script.
    pub script_name: Option<String>,
    /// Whether to send the generated images.
    pub send_images: Option<bool>,
    /// Whether to send the generated images.
    pub save_images: Option<bool>,
    /// Scripts to always run.
    pub alwayson_scripts: Option<HashMap<String, serde_json::Value>>,
}

impl Txt2ImgRequest {
    /// Adds a prompt to the request.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A String representing the prompt to be used for image generation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_prompt("A blue sky with green grass".to_string());
    /// ```
    pub fn with_prompt(&mut self, prompt: String) -> &mut Self {
        self.prompt = Some(prompt);
        self
    }

    /// Adds styles to the request.
    ///
    /// # Arguments
    ///
    /// * `styles` - A vector of Strings representing the styles to be used for image generation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_styles(vec!["cubism".to_string(), "impressionism".to_string()]);
    /// ```
    pub fn with_styles(&mut self, styles: Vec<String>) -> &mut Self {
        self.styles = Some(styles);
        self
    }

    /// Adds a style to the request.
    ///
    ///
    /// # Arguments
    ///
    /// * `style` - A String representing the style to be used for image generation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_style("cubism".to_string());
    /// ```
    pub fn with_style(&mut self, style: String) -> &mut Self {
        if let Some(ref mut styles) = self.styles {
            styles.push(style);
            self
        } else {
            self.with_styles(vec![style])
        }
    }

    /// Sets the seed for random number generation.
    ///
    /// # Arguments
    ///
    /// * `seed` - An i64 value representing the seed for random number generation.
    ///            Set to `-1` to randomize.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_seed(12345);
    /// ```
    pub fn with_seed(&mut self, seed: i64) -> &mut Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the subseed for random number generation.
    ///
    /// # Arguments
    ///
    /// * `subseed` - An i64 value representing the subseed for random number generation.
    ///               Set to `-1` to randomize.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_subseed(12345);
    /// ```
    pub fn with_subseed(&mut self, subseed: i64) -> &mut Self {
        self.subseed = Some(subseed);
        self
    }

    /// Sets the strength of the subseed parameter for image generation.
    ///
    /// # Arguments
    ///
    /// * `subseed_strength` - A u32 value representing the strength of the subseed parameter.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_subseed_strength(5);
    /// ```
    pub fn with_subseed_strength(&mut self, subseed_strength: u32) -> &mut Self {
        self.subseed_strength = Some(subseed_strength);
        self
    }

    /// Sets the sampler name for image generation.
    ///
    /// # Arguments
    ///
    /// * `sampler_name` - A String representing the sampler name to be used.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_sampler_name("Euler".to_string());
    /// ```
    pub fn with_sampler_name(&mut self, sampler_name: String) -> &mut Self {
        self.sampler_name = Some(sampler_name);
        self
    }

    /// Sets the batch size for image generation.
    ///
    /// # Arguments
    ///
    /// * `batch_size` - A u32 value representing the batch size to be used.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_batch_size(16);
    /// ```
    pub fn with_batch_size(&mut self, batch_size: u32) -> &mut Self {
        self.batch_size = Some(batch_size);
        self
    }

    /// Sets the number of iterations for image generation.
    ///
    /// # Arguments
    ///
    /// * `n_iter` - A u32 value representing the number of iterations to run for image generation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_n_iter(1000);
    /// ```
    pub fn with_n_iter(&mut self, n_iter: u32) -> &mut Self {
        self.n_iter = Some(n_iter);
        self
    }

    /// Sets the number of steps for image generation.
    ///
    /// # Arguments
    ///
    /// * `steps` - A u32 value representing the number of steps for image generation.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_steps(50);
    /// ```
    pub fn with_steps(&mut self, steps: u32) -> &mut Self {
        self.steps = Some(steps);
        self
    }

    /// Sets the cfg scale for image generation.
    ///
    /// # Arguments
    ///
    /// * `cfg_scale` - A f64 value representing the cfg scale parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_cfg_scale(0.7);
    /// ```
    pub fn with_cfg_scale(&mut self, cfg_scale: f64) -> &mut Self {
        self.cfg_scale = Some(cfg_scale);
        self
    }

    /// Sets the width for image generation.
    ///
    /// # Arguments
    ///
    /// * `width` - A u32 value representing the image width.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_width(512);
    /// ```
    pub fn with_width(&mut self, width: u32) -> &mut Self {
        self.width = Some(width);
        self
    }

    /// Sets the height for image generation.
    ///
    /// # Arguments
    ///
    /// * `height` - A u32 value representing the image height.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_height(512);
    /// ```
    pub fn with_height(&mut self, height: u32) -> &mut Self {
        self.height = Some(height);
        self
    }

    /// Enable or disable face restoration.
    ///
    /// # Arguments
    ///
    /// * `restore_faces` - A bool value to enable or disable face restoration.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_restore_faces(true);
    /// ```
    pub fn with_restore_faces(&mut self, restore_faces: bool) -> &mut Self {
        self.restore_faces = Some(restore_faces);
        self
    }

    /// Enable or disable image tiling.
    ///
    /// # Arguments
    ///
    /// * `tiling` - A bool value to enable or disable tiling.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_tiling(true);
    /// ```
    pub fn with_tiling(&mut self, tiling: bool) -> &mut Self {
        self.tiling = Some(tiling);
        self
    }

    /// Adds a negative prompt to the request.
    ///
    /// # Arguments
    ///
    /// * `negative_prompt` - A String representing the negative prompt to be used for image generation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Txt2ImgRequest::default();
    /// req.with_prompt("bad, ugly, worst quality".to_string());
    /// ```
    pub fn with_negative_prompt(&mut self, negative_prompt: String) -> &mut Self {
        self.negative_prompt = Some(negative_prompt);
        self
    }

    /// Merges the given settings with the request's settings.
    ///
    /// # Arguments
    ///
    /// * `request` - A Txt2ImgRequest containing the settings to merge.
    pub fn merge(&self, request: Self) -> Self {
        Self {
            enable_hr: request.enable_hr.or(self.enable_hr),
            denoising_strength: request.denoising_strength.or(self.denoising_strength),
            firstphase_width: request.firstphase_width.or(self.firstphase_width),
            firstphase_height: request.firstphase_height.or(self.firstphase_height),
            hr_scale: request.hr_scale.or(self.hr_scale),
            hr_upscaler: request.hr_upscaler.or(self.hr_upscaler.clone()),
            hr_second_pass_steps: request.hr_second_pass_steps.or(self.hr_second_pass_steps),
            hr_resize_x: request.hr_resize_x.or(self.hr_resize_x),
            hr_resize_y: request.hr_resize_y.or(self.hr_resize_y),
            prompt: request.prompt.or(self.prompt.clone()),
            styles: request.styles.or(self.styles.clone()),
            seed: request.seed.or(self.seed),
            subseed: request.subseed.or(self.subseed),
            subseed_strength: request.subseed_strength.or(self.subseed_strength),
            seed_resize_from_h: request.seed_resize_from_h.or(self.seed_resize_from_h),
            seed_resize_from_w: request.seed_resize_from_w.or(self.seed_resize_from_w),
            sampler_name: request.sampler_name.or(self.sampler_name.clone()),
            batch_size: request.batch_size.or(self.batch_size),
            n_iter: request.n_iter.or(self.n_iter),
            steps: request.steps.or(self.steps),
            cfg_scale: request.cfg_scale.or(self.cfg_scale),
            width: request.width.or(self.width),
            height: request.height.or(self.height),
            restore_faces: request.restore_faces.or(self.restore_faces),
            tiling: request.tiling.or(self.tiling),
            do_not_save_samples: request.do_not_save_samples.or(self.do_not_save_samples),
            do_not_save_grid: request.do_not_save_grid.or(self.do_not_save_grid),
            negative_prompt: request.negative_prompt.or(self.negative_prompt.clone()),
            eta: request.eta.or(self.eta),
            s_churn: request.s_churn.or(self.s_churn),
            s_tmax: request.s_tmax.or(self.s_tmax),
            s_tmin: request.s_tmin.or(self.s_tmin),
            s_noise: request.s_noise.or(self.s_noise),
            override_settings: request.override_settings.or(self.override_settings.clone()),
            override_settings_restore_afterwards: request
                .override_settings_restore_afterwards
                .or(self.override_settings_restore_afterwards),
            script_args: request.script_args.or(self.script_args.clone()),
            sampler_index: request.sampler_index.or(self.sampler_index.clone()),
            script_name: request.script_name.or(self.script_name.clone()),
            send_images: request.send_images.or(self.send_images),
            save_images: request.save_images.or(self.save_images),
            alwayson_scripts: request.alwayson_scripts.or(self.alwayson_scripts.clone()),
        }
    }
}

/// A client for sending image requests to a specified endpoint.
pub struct Txt2Img {
    client: reqwest::Client,
    endpoint: Url,
}

impl Txt2Img {
    /// Constructs a new Txt2Img client with a given `reqwest::Client` and Stable Diffusion API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new Txt2Img instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
    }

    /// Constructs a new Txt2Img client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new Txt2Img instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Sends an image request using the Txt2Img client.
    ///
    /// # Arguments
    ///
    /// * `request` - An Txt2ImgRequest containing the parameters for the image request.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ImgResponse<Txt2ImgRequest>` on success, or an error if one occurred.
    pub async fn send(
        &self,
        request: &Txt2ImgRequest,
    ) -> anyhow::Result<ImgResponse<Txt2ImgRequest>> {
        self.client
            .post(self.endpoint.clone())
            .json(&request)
            .send()
            .await
            .context("failed to send request")?
            .json()
            .await
            .context("failed to parse json")
    }
}
