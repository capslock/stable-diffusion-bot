# Stable Diffusion API

## Description

The `stable-diffusion-api` crate provides a basic API to interact with the backend API of the
[Stable Diffusion web UI](https://github.com/AUTOMATIC1111/stable-diffusion-webui).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
stable-diffusion-api = "0.1.2"
```

## Usage

To use, create an `Api` object, which can then be used to talk to the `Txt2Img`
or `Img2Img` endpoints. Then configure a `Txt2ImgRequest` or `Img2ImgRequest`
with your desired generation parameters:

```rust
use stable_diffusion_api::*;

let api = Api::default();
let txt2img = api.txt2img()?;
let mut request = Txt2ImgRequest::default();
request.with_prompt("a watercolor of a corgi wearing a tophat".to_string());
let resp = txt2img.send(&request).await?;
```
