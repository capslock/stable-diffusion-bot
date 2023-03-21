mod txt2img;
use anyhow::Context;
use reqwest::Url;
pub use txt2img::*;

mod img2img;
pub use img2img::*;

pub struct Api {
    client: reqwest::Client,
    url: Url,
}

impl Default for Api {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            url: Url::parse("http://localhost:7860").expect("Failed to parse default URL"),
        }
    }
}

impl Api {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_url<S>(url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            url: Url::parse(url.as_ref()).context("Failed to parse URL")?,
            ..Default::default()
        })
    }

    pub fn new_with_client_and_url<S>(client: reqwest::Client, url: S) -> anyhow::Result<Self>
    where
        S: AsRef<str>,
    {
        Ok(Self {
            client,
            url: Url::parse(url.as_ref()).context("Failed to parse URL")?,
        })
    }

    pub fn txt2img(&self) -> anyhow::Result<Txt2Img> {
        Ok(Txt2Img::new_with_url(
            self.client.clone(),
            self.url
                .join("sdapi/v1/txt2img")
                .context("Failed to parse txt2img endpoint")?,
        ))
    }

    pub fn img2img(&self) -> anyhow::Result<Img2Img> {
        Ok(Img2Img::new_with_url(
            self.client.clone(),
            self.url
                .join("sdapi/v1/img2img")
                .context("Failed to parse img2img endpoint")?,
        ))
    }
}
