use comfyui_api::models::AsAny;
use dyn_clone::DynClone;
use stable_diffusion_api::ImgInfo;

dyn_clone::clone_trait_object!(ImageParams);

pub trait ImageParams: std::fmt::Debug + AsAny + Send + Sync + DynClone {
    fn seed(&self) -> Option<i64>;
    fn steps(&self) -> Option<u32>;
    fn cfg(&self) -> Option<f32>;
    fn width(&self) -> Option<u32>;
    fn height(&self) -> Option<u32>;
    fn prompt(&self) -> Option<String>;
    fn negative_prompt(&self) -> Option<String>;
    fn denoising(&self) -> Option<f32>;
    fn model(&self) -> Option<String>;
    fn sampler(&self) -> Option<String>;
}

impl ImageParams for comfyui_api::models::Prompt {
    fn seed(&self) -> Option<i64> {
        comfyui_api::comfy::getter::SeedExt::seed(self)
            .ok()
            .copied()
    }

    fn steps(&self) -> Option<u32> {
        comfyui_api::comfy::getter::StepsExt::steps(self)
            .ok()
            .copied()
    }

    fn cfg(&self) -> Option<f32> {
        comfyui_api::comfy::getter::CfgExt::cfg(self).ok().copied()
    }

    fn width(&self) -> Option<u32> {
        comfyui_api::comfy::getter::WidthExt::width(self)
            .ok()
            .copied()
    }

    fn height(&self) -> Option<u32> {
        comfyui_api::comfy::getter::HeightExt::height(self)
            .ok()
            .copied()
    }

    fn prompt(&self) -> Option<String> {
        comfyui_api::comfy::getter::PromptExt::prompt(self)
            .ok()
            .cloned()
    }

    fn negative_prompt(&self) -> Option<String> {
        comfyui_api::comfy::getter::NegativePromptExt::negative_prompt(self)
            .ok()
            .cloned()
    }

    fn denoising(&self) -> Option<f32> {
        comfyui_api::comfy::getter::DenoiseExt::denoise(self)
            .ok()
            .copied()
    }

    fn model(&self) -> Option<String> {
        comfyui_api::comfy::getter::ModelExt::ckpt_name(self)
            .ok()
            .cloned()
    }

    fn sampler(&self) -> Option<String> {
        comfyui_api::comfy::getter::SamplerExt::sampler_name(self)
            .ok()
            .cloned()
    }
}

impl ImageParams for ImgInfo {
    fn seed(&self) -> Option<i64> {
        self.seed
    }

    fn steps(&self) -> Option<u32> {
        self.steps
    }

    fn cfg(&self) -> Option<f32> {
        self.cfg_scale.map(|c| c as f32)
    }

    fn width(&self) -> Option<u32> {
        self.width.map(|w| w as u32)
    }

    fn height(&self) -> Option<u32> {
        self.height.map(|h| h as u32)
    }

    fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt.clone()
    }

    fn denoising(&self) -> Option<f32> {
        self.denoising_strength.map(|d| d as f32)
    }

    fn model(&self) -> Option<String> {
        self.sd_model_name.clone()
    }

    fn sampler(&self) -> Option<String> {
        self.sampler_name.clone()
    }
}
