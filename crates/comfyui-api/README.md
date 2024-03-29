# ComfyUI API

## Description

The `comfyui-api` crate provides a basic API to interact with the backend API of the
[ComfyUI](https://github.com/comfyanonymous/ComfyUI) Stable Diffusion GUI.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
comfyui-api = "0.1.0"
```

## Usage

### Low-level API

The low-level API directly wraps the ComfyUI API. This API is available in the
[`api`](https://capslock.github.io/stable-diffusion-bot/comfyui_api/api/index.html)
module, with the models in the
[`models`](https://capslock.github.io/stable-diffusion-bot/comfyui_api/models/index.html)
module.

```rust
use comfyui_api::{api::*, models::*};
```

### High-level API

The high-level API wraps the low-level API with simplified functions that are
well-suited to directly running a prompt and fetching the resulting image. This
API is available in the
[`comfy`](https://capslock.github.io/stable-diffusion-bot/comfyui_api/comfy/index.html)
module, and uses types from the 
[`models`](https://capslock.github.io/stable-diffusion-bot/comfyui_api/models/index.html)
module.

```rust
use comfyui_api::comfy::*;
```

### Usage Examples

See `examples/simple.rs` for a full-featured usage example of the low-level API.
See `examples/comfy.rs` for an example of using the higher-level Comfy API.
