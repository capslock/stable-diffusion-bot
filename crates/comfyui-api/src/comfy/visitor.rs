use crate::models::*;

/// Trait for visiting nodes in a ComfyUI graph.
pub trait Visitor {
    /// Returns a reference to the node with the given id.
    fn get_node_by_id(&self, id: &str) -> &NodeOrUnknown;

    /// Visits a node or unknown node. This is the entry point for the visitor.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to visit.
    fn visit_node_or_unknown(&mut self, node: &NodeOrUnknown) {
        walk_node_or_unknown(self, node);
    }

    /// Visits a node.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to visit.
    fn visit_node(&mut self, node: &Node) {
        walk_node(self, node);
    }

    /// Visits a generic node.
    ///
    /// # Arguments
    ///
    /// * `node` - The generic node to visit.
    fn visit_generic_node(&mut self, node: &GenericNode) {
        walk_generic_node(self, node);
    }

    /// Visits a KSampler node.
    ///
    /// # Arguments
    ///
    /// * `node` - The KSampler node to visit.
    fn visit_k_sampler(&mut self, node: &KSampler) {
        walk_k_sampler(self, node);
    }

    /// Visits a CLIPTextEncode node.
    ///
    /// # Arguments
    ///
    /// * `node` - The CLIPTextEncode node to visit.
    fn visit_clip_text_encode(&mut self, node: &CLIPTextEncode) {
        walk_clip_text_encode(self, node);
    }

    /// Visits an EmptyLatentImage node.
    ///
    /// # Arguments
    ///
    /// * `node` - The EmptyLatentImage node to visit.
    fn visit_empty_latent_image(&mut self, node: &EmptyLatentImage) {
        walk_empty_latent_image(self, node);
    }

    /// Visits a CheckpointLoaderSimple node.
    ///
    /// # Arguments
    ///
    /// * `node` - The CheckpointLoaderSimple node to visit.
    fn visit_checkpoint_loader_simple(&mut self, node: &CheckpointLoaderSimple) {
        walk_checkpoint_loader_simple(self, node);
    }

    /// Visits a VAELoader node.
    ///
    /// # Arguments
    ///
    /// * `node` - The VAELoader node to visit.
    fn visit_vae_loader(&mut self, node: &VAELoader) {
        walk_vae_loader(self, node);
    }

    /// Visits a VAEDecode node.
    ///
    /// # Arguments
    ///
    /// * `node` - The VAEDecode node to visit.
    fn visit_vae_decode(&mut self, node: &VAEDecode) {
        walk_vae_decode(self, node);
    }

    /// Visits a PreviewImage node.
    ///
    /// # Arguments
    ///
    /// * `node` - The PreviewImage node to visit.
    fn visit_preview_image(&mut self, node: &PreviewImage) {
        walk_preview_image(self, node);
    }

    /// Visits a KSamplerSelect node.
    ///
    /// # Arguments
    ///
    /// * `node` - The KSamplerSelect node to visit.
    fn visit_k_sampler_select(&mut self, node: &KSamplerSelect) {
        walk_k_sampler_select(self, node);
    }

    /// Visits a SamplerCustom node.
    ///
    /// # Arguments
    ///
    /// * `node` - The SamplerCustom node to visit.
    fn visit_sampler_custom(&mut self, node: &SamplerCustom) {
        walk_sampler_custom(self, node);
    }

    /// Visits a SDTurboScheduler node.
    ///
    /// # Arguments
    ///
    /// * `node` - The SDTurboScheduler node to visit.
    fn visit_sd_turbo_scheduler(&mut self, node: &SDTurboScheduler) {
        walk_sd_turbo_scheduler(self, node);
    }

    /// Visits a ImageOnlyCheckpointLoader node.
    ///
    /// # Arguments
    ///
    /// * `node` - The ImageOnlyCheckpointLoader node to visit.
    fn visit_image_only_checkpoint_loader(&mut self, node: &ImageOnlyCheckpointLoader) {
        walk_image_only_checkpoint_loader(self, node);
    }

    /// Visits a LoadImage node.
    ///
    /// # Arguments
    ///
    /// * `node` - The LoadImage node to visit.
    fn visit_load_image(&mut self, node: &LoadImage) {
        walk_load_image(self, node);
    }

    /// Visits a SVDimg2vidConditioning node.
    ///
    /// # Arguments
    ///
    /// * `node` - The SVDimg2vidConditioning node to visit.
    fn visit_svdimg2vid_conditioning(&mut self, node: &SVDimg2vidConditioning) {
        walk_svdimg2vid_conditioning(self, node);
    }

    /// Visits a VideoLinearCFGGuidance node.
    ///
    /// # Arguments
    ///
    /// * `node` - The VideoLinearCFGGuidance node to visit.
    fn visit_video_linear_cfg_guidance(&mut self, node: &VideoLinearCFGGuidance) {
        walk_video_linear_cfg_guidance(self, node);
    }

    /// Visits a SaveAnimatedWEBP node.
    ///
    /// # Arguments
    ///
    /// * `node` - The SaveAnimatedWEBP node to visit.
    fn visit_save_animated_webp(&mut self, node: &SaveAnimatedWEBP) {
        walk_save_animated_webp(self, node);
    }

    /// Visits a LoraLoader node.
    ///
    /// # Arguments
    ///
    /// * `node` - The LoraLoader node to visit.
    fn visit_lora_loader(&mut self, node: &LoraLoader) {
        walk_lora_loader(self, node);
    }

    /// Visits a ModelSamplingDiscrete node.
    ///
    /// # Arguments
    ///
    /// * `node` - The ModelSamplingDiscrete node to visit.
    fn visit_model_sampling_discrete(&mut self, node: &ModelSamplingDiscrete) {
        walk_model_sampling_discrete(self, node);
    }

    /// Visits a SaveImage node.
    ///
    /// # Arguments
    ///
    /// * `node` - The SaveImage node to visit.
    fn visit_save_image(&mut self, node: &SaveImage) {
        walk_save_image(self, node);
    }

    /// Visits an input.
    ///
    /// # Arguments
    ///
    /// * `input` - The input to visit.
    fn visit_input<T>(&mut self, input: &Input<T>) {
        walk_input(self, input);
    }
}

/// Traverse a node or unknown node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `node` - The node to traverse.
pub fn walk_node_or_unknown<V: Visitor + ?Sized>(visitor: &mut V, node: &NodeOrUnknown) {
    match node {
        NodeOrUnknown::Node(node) => visitor.visit_node(node),
        NodeOrUnknown::GenericNode(node) => visitor.visit_generic_node(node),
    }
}

/// Traverse a node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `node` - The node to traverse.
pub fn walk_node<V: Visitor + ?Sized>(visitor: &mut V, node: &Node) {
    match *node {
        Node::KSampler(ref node) => visitor.visit_k_sampler(node),
        Node::CLIPTextEncode(ref node) => visitor.visit_clip_text_encode(node),
        Node::EmptyLatentImage(ref node) => visitor.visit_empty_latent_image(node),
        Node::CheckpointLoaderSimple(ref node) => visitor.visit_checkpoint_loader_simple(node),
        Node::VAELoader(ref node) => visitor.visit_vae_loader(node),
        Node::VAEDecode(ref node) => visitor.visit_vae_decode(node),
        Node::PreviewImage(ref node) => visitor.visit_preview_image(node),
        Node::KSamplerSelect(ref node) => visitor.visit_k_sampler_select(node),
        Node::SamplerCustom(ref node) => visitor.visit_sampler_custom(node),
        Node::SDTurboScheduler(ref node) => visitor.visit_sd_turbo_scheduler(node),
        Node::ImageOnlyCheckpointLoader(ref node) => {
            visitor.visit_image_only_checkpoint_loader(node)
        }
        Node::LoadImage(ref node) => visitor.visit_load_image(node),
        Node::SVDimg2vidConditioning(ref node) => visitor.visit_svdimg2vid_conditioning(node),
        Node::VideoLinearCFGGuidance(ref node) => visitor.visit_video_linear_cfg_guidance(node),
        Node::SaveAnimatedWEBP(ref node) => visitor.visit_save_animated_webp(node),
        Node::LoraLoader(ref node) => visitor.visit_lora_loader(node),
        Node::ModelSamplingDiscrete(ref node) => visitor.visit_model_sampling_discrete(node),
        Node::SaveImage(ref node) => visitor.visit_save_image(node),
    }
}

/// Traverse a generic node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `node` - The generic node to traverse.
pub fn walk_generic_node<V: Visitor + ?Sized>(visitor: &mut V, n: &GenericNode) {
    for (_, value) in n.inputs.iter() {
        if let GenericValue::NodeConnection(node_connection) = value {
            visitor
                .visit_node_or_unknown(&visitor.get_node_by_id(&node_connection.node_id).clone());
        }
    }
}

/// Traverse an input.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `input` - The input to traverse.
pub fn walk_input<V: Visitor + ?Sized, T>(visitor: &mut V, input: &Input<T>) {
    if let Input::NodeConnection(node_connection) = input {
        visitor.visit_node_or_unknown(&visitor.get_node_by_id(&node_connection.node_id).clone());
    }
}

/// Traverse a KSampler node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The KSampler node to traverse.
pub fn walk_k_sampler<V: Visitor + ?Sized>(visitor: &mut V, n: &KSampler) {
    visitor.visit_input(&n.cfg);
    visitor.visit_input(&n.denoise);
    visitor.visit_input(&n.sampler_name);
    visitor.visit_input(&n.scheduler);
    visitor.visit_input(&n.seed);
    visitor.visit_input(&n.steps);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.positive.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.negative.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.latent_image.node_id).clone());
}

/// Traverse a CLIPTextEncode node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The CLIPTextEncode node to traverse.
pub fn walk_clip_text_encode<V: Visitor + ?Sized>(visitor: &mut V, n: &CLIPTextEncode) {
    visitor.visit_input(&n.text);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.clip.node_id).clone());
}

/// Traverse an EmptyLatentImage node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The EmptyLatentImage node to traverse.
pub fn walk_empty_latent_image<V: Visitor + ?Sized>(visitor: &mut V, n: &EmptyLatentImage) {
    visitor.visit_input(&n.width);
    visitor.visit_input(&n.height);
}

/// Traverse a CheckpointLoaderSimple node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The CheckpointLoaderSimple node to traverse.
pub fn walk_checkpoint_loader_simple<V: Visitor + ?Sized>(
    visitor: &mut V,
    n: &CheckpointLoaderSimple,
) {
    visitor.visit_input(&n.ckpt_name);
}

/// Traverse a VAELoader node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The VAELoader node to traverse.
pub fn walk_vae_loader<V: Visitor + ?Sized>(visitor: &mut V, n: &VAELoader) {
    visitor.visit_input(&n.vae_name);
}

/// Traverse a VAEDecode node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The VAEDecode node to traverse.
pub fn walk_vae_decode<V: Visitor + ?Sized>(visitor: &mut V, n: &VAEDecode) {
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.vae.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.samples.node_id).clone());
}

/// Traverse a PreviewImage node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The PreviewImage node to traverse.
pub fn walk_preview_image<V: Visitor + ?Sized>(visitor: &mut V, n: &PreviewImage) {
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.images.node_id).clone());
}

/// Traverse a KSamplerSelect node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The KSamplerSelect node to traverse.
pub fn walk_k_sampler_select<V: Visitor + ?Sized>(visitor: &mut V, n: &KSamplerSelect) {
    visitor.visit_input(&n.sampler_name);
}

/// Traverse a SamplerCustom node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The SamplerCustom node to traverse.
pub fn walk_sampler_custom<V: Visitor + ?Sized>(visitor: &mut V, n: &SamplerCustom) {
    visitor.visit_input(&n.add_noise);
    visitor.visit_input(&n.cfg);
    visitor.visit_input(&n.noise_seed);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.positive.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.negative.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.latent_image.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.sampler.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.sigmas.node_id).clone());
}

/// Traverse a SDTurboScheduler node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The SDTurboScheduler node to traverse.
pub fn walk_sd_turbo_scheduler<V: Visitor + ?Sized>(visitor: &mut V, n: &SDTurboScheduler) {
    visitor.visit_input(&n.steps);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
}

/// Traverse a ImageOnlyCheckpointLoader node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The ImageOnlyCheckpointLoader node to traverse.
pub fn walk_image_only_checkpoint_loader<V: Visitor + ?Sized>(
    visitor: &mut V,
    n: &ImageOnlyCheckpointLoader,
) {
    visitor.visit_input(&n.ckpt_name);
}

/// Traverse a LoadImage node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The LoadImage node to traverse.
pub fn walk_load_image<V: Visitor + ?Sized>(visitor: &mut V, n: &LoadImage) {
    visitor.visit_input(&n.file_to_upload);
    visitor.visit_input(&n.image);
}

/// Traverse a SVDimg2vidConditioning node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The SVDimg2vidConditioning node to traverse.
pub fn walk_svdimg2vid_conditioning<V: Visitor + ?Sized>(
    visitor: &mut V,
    n: &SVDimg2vidConditioning,
) {
    visitor.visit_input(&n.augmentation_level);
    visitor.visit_input(&n.fps);
    visitor.visit_input(&n.width);
    visitor.visit_input(&n.height);
    visitor.visit_input(&n.motion_bucket_id);
    visitor.visit_input(&n.video_frames);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.clip_vision.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.init_image.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.vae.node_id).clone());
}

/// Traverse a VideoLinearCFGGuidance node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The VideoLinearCFGGuidance node to traverse.
pub fn walk_video_linear_cfg_guidance<V: Visitor + ?Sized>(
    visitor: &mut V,
    n: &VideoLinearCFGGuidance,
) {
    visitor.visit_input(&n.min_cfg);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
}

/// Traverse a SaveAnimatedWEBP node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The SaveAnimatedWEBP node to traverse.
pub fn walk_save_animated_webp<V: Visitor + ?Sized>(visitor: &mut V, n: &SaveAnimatedWEBP) {
    visitor.visit_input(&n.filename_prefix);
    visitor.visit_input(&n.fps);
    visitor.visit_input(&n.lossless);
    visitor.visit_input(&n.method);
    visitor.visit_input(&n.quality);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.images.node_id).clone());
}

/// Traverse a LoraLoader node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The LoraLoader node to traverse.
pub fn walk_lora_loader<V: Visitor + ?Sized>(visitor: &mut V, n: &LoraLoader) {
    visitor.visit_input(&n.lora_name);
    visitor.visit_input(&n.strength_model);
    visitor.visit_input(&n.strength_clip);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.clip.node_id).clone());
}

/// Traverse a ModelSamplingDiscrete node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The ModelSamplingDiscrete node to traverse.
pub fn walk_model_sampling_discrete<V: Visitor + ?Sized>(
    visitor: &mut V,
    n: &ModelSamplingDiscrete,
) {
    visitor.visit_input(&n.sampling);
    visitor.visit_input(&n.zsnr);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.model.node_id).clone());
}

/// Traverse a SaveImage node.
///
/// # Arguments
///
/// * `visitor` - The visitor to use.
/// * `n` - The SaveImage node to traverse.
pub fn walk_save_image<V: Visitor + ?Sized>(visitor: &mut V, n: &SaveImage) {
    visitor.visit_input(&n.filename_prefix);
    visitor.visit_node_or_unknown(&visitor.get_node_by_id(&n.images.node_id).clone());
}
