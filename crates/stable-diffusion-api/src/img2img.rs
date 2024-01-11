use std::collections::HashMap;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::ImgResponse;

/// Struct representing an image to image request.
#[skip_serializing_none]
#[derive(Default, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct Img2ImgRequest {
    /// Initial images.
    pub init_images: Option<Vec<String>>,
    /// Resize mode.
    pub resize_mode: Option<u32>,
    /// Strength of denoising applied to the image.
    pub denoising_strength: Option<f64>,
    /// CFG scale.
    pub image_cfg_scale: Option<u32>,
    /// Mask.
    pub mask: Option<String>,
    /// Blur to apply to the mask.
    pub mask_blur: Option<u32>,
    /// Amount of inpainting to apply.
    pub inpainting_fill: Option<u32>,
    /// Whether to perform inpainting at full resolution.
    pub inpaint_full_res: Option<bool>,
    /// Padding to apply when performing inpainting at full resolution.
    pub inpaint_full_res_padding: Option<u32>,
    /// Whether to invert the inpainting mask.
    pub inpainting_mask_invert: Option<u32>,
    /// Initial noise multiplier.
    pub initial_noise_multiplier: Option<u32>,
    /// Text prompt for generating the image.
    pub prompt: Option<String>,
    /// List of style prompts for generating the image.
    pub styles: Option<Vec<String>>,
    /// Seed.
    pub seed: Option<i64>,
    /// Subseed.
    pub subseed: Option<i64>,
    /// Strength of the subseed.
    pub subseed_strength: Option<u32>,
    /// Height to resize the seed image from.
    pub seed_resize_from_h: Option<i32>,
    /// Width to resize the seed image from.
    pub seed_resize_from_w: Option<i32>,
    /// Name of the sampler.
    pub sampler_name: Option<String>,
    /// Batch size.
    pub batch_size: Option<u32>,
    /// Number of iterations.
    pub n_iter: Option<u32>,
    /// Number of steps.
    pub steps: Option<u32>,
    /// CFG scale.
    pub cfg_scale: Option<f64>,
    /// Width of the generated image.
    pub width: Option<u32>,
    /// Height of the generated image.
    pub height: Option<u32>,
    /// Whether to restore faces in the generated image.
    pub restore_faces: Option<bool>,
    /// Whether tiling for the generated image.
    pub tiling: Option<bool>,
    /// Whether to save samples when generating multiple images.
    pub do_not_save_samples: Option<bool>,
    /// Whether to save the grid when generating multiple images.
    pub do_not_save_grid: Option<bool>,
    /// Negative prompt.
    pub negative_prompt: Option<String>,
    /// Eta value.
    pub eta: Option<u32>,
    /// Churn value.
    pub s_churn: Option<f64>,
    /// Maximum temperature value.
    pub s_tmax: Option<f64>,
    /// Minimum temperature value.
    pub s_tmin: Option<f64>,
    /// Amount of noise.
    pub s_noise: Option<f64>,
    /// Settings to override when generating the image.
    pub override_settings: Option<HashMap<String, serde_json::Value>>,
    /// Whether to restore the settings after generating the image.
    pub override_settings_restore_afterwards: Option<bool>,
    /// Arguments to pass to the script.
    pub script_args: Option<Vec<serde_json::Value>>,
    /// Index of the sampler.
    pub sampler_index: Option<String>,
    /// Whether to include initial images in the output.
    pub include_init_images: Option<bool>,
    /// Name of the script.
    pub script_name: Option<String>,
    /// Whether to send the generated images.
    pub send_images: Option<bool>,
    /// Whether to save the generated images.
    pub save_images: Option<bool>,
    /// Scripts to always run.
    pub alwayson_scripts: Option<HashMap<String, serde_json::Value>>,
}

impl Img2ImgRequest {
    /// Adds a prompt to the request.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A String representing the prompt to be used for image generation.
    ///
    /// # Example
    ///
    /// ```
    /// let mut req = Img2ImgRequest::default();
    /// req.with_prompt("A blue sky with green grass".to_string());
    /// ```
    pub fn with_prompt(&mut self, prompt: String) -> &mut Self {
        self.prompt = Some(prompt);
        self
    }

    /// Adds a single image to the request.
    ///
    /// # Arguments
    ///
    /// * `image` - array bytes of the image to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// let mut req = Img2ImgRequest::default();
    /// let image_data = fs::read("path/to/image.jpg").unwrap();
    /// req.with_image(image_data);
    /// ```
    pub fn with_image<T>(&mut self, image: T) -> &mut Self
    where
        T: AsRef<[u8]>,
    {
        use base64::{engine::general_purpose, Engine as _};

        if let Some(ref mut images) = self.init_images {
            images.push(general_purpose::STANDARD.encode(image));
            self
        } else {
            self.with_images(vec![image])
        }
    }

    /// Adds multiple images to the request.
    ///
    /// # Arguments
    ///
    /// * `images` - A vector of byte arrays that represents the images to be added.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// let mut req = Img2ImgRequest::default();
    /// let image_data1 = fs::read("path/to/image1.jpg").unwrap();
    /// let image_data2 = fs::read("path/to/image2.jpg").unwrap();
    /// req.with_images(vec![image_data1, image_data2]);
    /// ```
    pub fn with_images<T>(&mut self, images: Vec<T>) -> &mut Self
    where
        T: AsRef<[u8]>,
    {
        use base64::{engine::general_purpose, Engine as _};
        let images = images.iter().map(|i| general_purpose::STANDARD.encode(i));
        if let Some(ref mut i) = self.init_images {
            i.extend(images);
        } else {
            self.init_images = Some(images.collect())
        }
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
    /// let mut req = Img2ImgRequest::default();
    /// req.with_styles(vec!["cubism".to_string(), "impressionism".to_string()]);
    /// ```
    pub fn with_styles(&mut self, styles: Vec<String>) -> &mut Self {
        if let Some(ref mut s) = self.styles {
            s.extend(styles);
        } else {
            self.styles = Some(styles);
        }
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
    /// let mut req = Img2ImgRequest::default();
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

    /// Sets the denoising strength for image generation.
    ///
    /// # Arguments
    ///
    /// * `denoising_strength` - A f64 value representing the denoising strength parameter.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut req = Img2ImgRequest::default();
    /// req.with_denoising_strength(0.4);
    /// ```
    pub fn with_denoising_strength(&mut self, denoising_strength: f64) -> &mut Self {
        self.denoising_strength = Some(denoising_strength);
        self
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// let mut req = Img2ImgRequest::default();
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
    /// * `request` - A Img2ImgRequest containing the settings to merge.
    pub fn merge(&self, request: Self) -> Self {
        Self {
            init_images: request.init_images.or(self.init_images.clone()),
            resize_mode: request.resize_mode.or(self.resize_mode),
            denoising_strength: request.denoising_strength.or(self.denoising_strength),
            image_cfg_scale: request.image_cfg_scale.or(self.image_cfg_scale),
            mask: request.mask.or(self.mask.clone()),
            mask_blur: request.mask_blur.or(self.mask_blur),
            inpainting_fill: request.inpainting_fill.or(self.inpainting_fill),
            inpaint_full_res: request.inpaint_full_res.or(self.inpaint_full_res),
            inpaint_full_res_padding: request
                .inpaint_full_res_padding
                .or(self.inpaint_full_res_padding),
            inpainting_mask_invert: request
                .inpainting_mask_invert
                .or(self.inpainting_mask_invert),
            initial_noise_multiplier: request
                .initial_noise_multiplier
                .or(self.initial_noise_multiplier),
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
            include_init_images: request.include_init_images.or(self.include_init_images),
            script_name: request.script_name.or(self.script_name.clone()),
            send_images: request.send_images.or(self.send_images),
            save_images: request.save_images.or(self.save_images),
            alwayson_scripts: request.alwayson_scripts.or(self.alwayson_scripts.clone()),
        }
    }
}

/// Errors that can occur when interacting with the `Img2Img` API.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Img2ImgError {
    /// Error parsing endpoint URL
    #[error("Failed to parse endpoint URL")]
    ParseError(#[from] url::ParseError),
    /// Error sending request
    #[error("Failed to send request")]
    RequestFailed(#[from] reqwest::Error),
    /// An error occurred while parsing the response from the API.
    #[error("Parsing response failed")]
    InvalidResponse(#[source] reqwest::Error),
    /// An error occurred getting response data.
    #[error("Failed to get response data")]
    GetDataFailed(#[source] reqwest::Error),
    /// Server returned an error for img2img
    #[error("Img2Img request failed: {status}: {error}")]
    Img2ImgFailed {
        status: reqwest::StatusCode,
        error: String,
    },
}

type Result<T> = std::result::Result<T, Img2ImgError>;

/// A client for sending image requests to a specified endpoint.
pub struct Img2Img {
    client: reqwest::Client,
    endpoint: Url,
}

impl Img2Img {
    /// Constructs a new Img2Img client with a given `reqwest::Client` and Stable Diffusion API
    /// endpoint `String`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `String` representation of the endpoint url.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new Img2Img instance on success, or an error if url parsing failed.
    pub fn new(client: reqwest::Client, endpoint: String) -> Result<Self> {
        Ok(Self::new_with_url(client, Url::parse(&endpoint)?))
    }

    /// Constructs a new Img2Img client with a given `reqwest::Client` and endpoint `Url`.
    ///
    /// # Arguments
    ///
    /// * `client` - A `reqwest::Client` used to send requests.
    /// * `endpoint` - A `Url` representing the endpoint url.
    ///
    /// # Returns
    ///
    /// A new Img2Img instance.
    pub fn new_with_url(client: reqwest::Client, endpoint: Url) -> Self {
        Self { client, endpoint }
    }

    /// Sends an image request using the Img2Img client.
    ///
    /// # Arguments
    ///
    /// * `request` - An Img2ImgRequest containing the parameters for the image request.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `ImgResponse<Img2ImgRequest>` on success, or an error if one occurred.
    pub async fn send(&self, request: &Img2ImgRequest) -> Result<ImgResponse<Img2ImgRequest>> {
        let response = self
            .client
            .post(self.endpoint.clone())
            .json(&request)
            .send()
            .await?;
        if response.status().is_success() {
            return response.json().await.map_err(Img2ImgError::InvalidResponse);
        }
        let status = response.status();
        let text = response.text().await.map_err(Img2ImgError::GetDataFailed)?;
        Err(Img2ImgError::Img2ImgFailed {
            status,
            error: text,
        })
    }
}
