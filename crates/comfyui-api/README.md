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
`api` module, with the models in the `models` module.

```rust
use comfyui_api::{api::*, models::*};
```

### High-level API

The high-level API wraps the low-level API with simplified functions that are
well-suited to directly running a prompt and fetching the resulting image. This
API is available in the `comfy` module, and uses types from the `models` module.

```rust
use comfyui_api::comfy::*;
```

## Usage

See `examples/simple.rs` for a full-featured usage example of the low-level API.
See `examples/comfy.rs` for an example of using the higher-level Comfy API.
