# SAL-E API

Stable-Diffusion Abstraction Layer

## Description

The `sal-e-api` crate provides an API abstraction layer, allowing the use of either a
[Stable Diffusion web UI](https://github.com/AUTOMATIC1111/stable-diffusion-webui) or
[ComfyUI](https://github.com/comfyanonymous/ComfyUI) backend for image generation.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sal-e-api = "0.1.0"
```

## Usage

This crate provides two main traits:

* [`Txt2ImgApi`](https://capslock.github.io/stable-diffusion-bot/sal_e_api/trait.Txt2ImgApi.html),
  which generates images from a text prompt,
* [`Img2ImgApi`](https://capslock.github.io/stable-diffusion-bot/sal_e_api/trait.Img2ImgApi.html),
  which generates images from a base image and a text prompt

There are two concrete implementations for each trait:

* [`StableDiffusionWebUiApi`](https://capslock.github.io/stable-diffusion-bot/sal_e_api/struct.StableDiffusionWebUiApi.html),
  which wraps the Stable Diffusion WebUI API,
* [`ComfyPromptApi`](https://capslock.github.io/stable-diffusion-bot/sal_e_api/struct.ComfyPromptApi.html),
  which wraps the ComfyUI API.

To use, create a concrete instance, set the generation parameters, and then call the generation function:

```rust
use sal_e_api::*;

let api = StableDiffusionWebUiApi::new();
let mut parameters = Txt2ImgParams::default();
parameters.set_prompt("a watercolor of a corgi wearing a tophat");
let result = api.txt2img(&parameters).await?;
```
