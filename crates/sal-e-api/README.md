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

Then in your code,

```rust
use sal_e_api::*;
```
