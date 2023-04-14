use std::collections::HashMap;

use anyhow::Context;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::ImgResponse;

/// Struct representing an image to image request.
#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
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
    pub seed: Option<i64>,
    pub subseed: Option<i64>,
    pub subseed_strength: Option<u32>,
    pub seed_resize_from_h: Option<i32>,
    pub seed_resize_from_w: Option<i32>,
    pub sampler_name: Option<String>,
    pub batch_size: Option<u32>,
    pub n_iter: Option<u32>,
    pub steps: Option<u32>,
    pub cfg_scale: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub restore_faces: Option<bool>,
    pub tiling: Option<bool>,
    pub do_not_save_samples: Option<bool>,
    pub do_not_save_grid: Option<bool>,
    pub negative_prompt: Option<String>,
    pub eta: Option<u32>,
    pub s_churn: Option<f64>,
    pub s_tmax: Option<f64>,
    pub s_tmin: Option<f64>,
    pub s_noise: Option<f64>,
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
}

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
    pub fn new(client: reqwest::Client, endpoint: String) -> anyhow::Result<Self> {
        Ok(Self::new_with_url(
            client,
            Url::parse(&endpoint).context("failed to parse endpoint url")?,
        ))
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
    pub async fn send(
        &self,
        request: &Img2ImgRequest,
    ) -> anyhow::Result<ImgResponse<Img2ImgRequest>> {
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
