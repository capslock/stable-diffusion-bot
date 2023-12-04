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

Then in your code,

```rust
use comfyui_api::*;
```

## Usage

See `examples/simple.rs` for a full-featured usage example of the low-level API.
See `examples/comfy.rs` for an example of using the higher-level Comfy API.

## License

This project is licensed under the MIT License - see the LICENSE.md file for details.