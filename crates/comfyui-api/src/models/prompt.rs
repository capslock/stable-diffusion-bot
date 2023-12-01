use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Struct representing a prompt workflow.
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Prompt {
    /// The prompt workflow, indexed by node id.
    #[serde(flatten)]
    pub workflow: HashMap<String, NodeOrUnknown>,
}

/// Enum capturing all possible node types.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NodeOrUnknown {
    /// Enum variant representing a known node.
    Node(Node),
    /// Variant capturing unknown nodes.
    GenericNode(GenericNode),
}

/// Enum of node types
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "class_type", content = "inputs")]
pub enum Node {
    /// Enum variant representing a KSampler node.
    KSampler(KSampler),
    /// Enum variant representing a CLIPTextEncode node.
    CLIPTextEncode(CLIPTextEncode),
    /// Enum variant representing an EmptyLatentImage node.
    EmptyLatentImage(EmptyLatentImage),
    /// Enum variant representing a CheckpointLoaderSimple node.
    CheckpointLoaderSimple(CheckpointLoaderSimple),
    /// Enum variant representing a VAELoader node.
    VAELoader(VAELoader),
    /// Enum variant representing a VAEDecode node.
    VAEDecode(VAEDecode),
    /// Enum variant representing a PreviewImage node.
    PreviewImage(PreviewImage),
    /// Enum variant representing a KSamplerSelect node.
    KSamplerSelect(KSamplerSelect),
    /// Enum variant representing a SamplerCustom node.
    SamplerCustom(SamplerCustom),
    /// Enum variant representing a SDTurboScheduler node.
    SDTurboScheduler(SDTurboScheduler),
    /// Enum variant representing a ImageOnlyCheckpointLoader node.
    ImageOnlyCheckpointLoader(ImageOnlyCheckpointLoader),
    /// Enum variant representing a LoadImage node.
    LoadImage(LoadImage),
    /// Enum variant representing a SVDimage2vidConditioning node.
    #[serde(rename = "SVD_img2vid_Conditioning")]
    SVDimg2vidConditioning(SVDimg2vidConditioning),
    /// Enum variant representing a VideoLinearCFGGuidance node.
    VideoLinearCFGGuidance(VideoLinearCFGGuidance),
    /// Enum variant representing a SaveAnimatedWEBP node.
    SaveAnimatedWEBP(SaveAnimatedWEBP),
    /// Enum variant representing a LoraLoader node.
    LoraLoader(LoraLoader),
    /// Enum variant representing a ModelSamplingDiscrete node.
    ModelSamplingDiscrete(ModelSamplingDiscrete),
    /// Enum variant representing a SaveImage node.
    SaveImage(SaveImage),
    // TODO: Implement other node types.
}

/// Struct representing a generic node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericNode {
    /// The node class type.
    pub class_type: String,
    /// The node inputs.
    pub inputs: HashMap<String, GenericValue>,
}

/// Enum of possible generic node input types.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GenericValue {
    /// Bool input variant.
    Bool(bool),
    /// Integer input variant.
    Int(i64),
    /// Float input variant.
    Float(f32),
    /// String input variant.
    String(String),
    /// Node connection input variant.
    NodeConnection(NodeConnection),
}

/// Struct representing a node input connection.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(String, u32)")]
#[serde(into = "(String, u32)")]
pub struct NodeConnection {
    /// The node id of the node providing the input.
    pub node_id: String,
    /// The index of the output from the node providing the input.
    pub output_index: u32,
}

impl From<(String, u32)> for NodeConnection {
    fn from((node_id, output_index): (String, u32)) -> Self {
        Self {
            node_id,
            output_index,
        }
    }
}

impl From<NodeConnection> for (String, u32) {
    fn from(
        NodeConnection {
            node_id,
            output_index,
        }: NodeConnection,
    ) -> Self {
        (node_id, output_index)
    }
}

/// Enum of inputs to a node.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Input<T> {
    /// Node connection input variant.
    NodeConnection(NodeConnection),
    /// Widget input variant.
    Value(T),
}

/// Struct representing a KSampler node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSampler {
    /// The cfg scale parameter.
    pub cfg: Input<f32>,
    /// The denoise parameter.
    pub denoise: Input<f32>,
    /// The sampler name.
    pub sampler_name: Input<String>,
    /// The scheduler used.
    pub scheduler: Input<String>,
    /// The seed.
    pub seed: Input<i64>,
    /// The number of steps.
    pub steps: Input<i32>,
    /// The positive conditioning input connection.
    pub positive: NodeConnection,
    /// The negative conditioning input connection.
    pub negative: NodeConnection,
    /// The model input connection.
    pub model: NodeConnection,
    /// The latent image input connection.
    pub latent_image: NodeConnection,
}

/// Struct representing a CLIPTextEncode node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CLIPTextEncode {
    /// The text to encode.
    pub text: Input<String>,
    /// The CLIP model input connection.
    pub clip: NodeConnection,
}

/// Struct representing an EmptyLatentImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyLatentImage {
    /// The batch size.
    pub batch_size: Input<u32>,
    /// The image width.
    pub width: Input<u32>,
    /// The image height.
    pub height: Input<u32>,
}

/// Struct representing a CheckpointLoaderSimple node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointLoaderSimple {
    /// The checkpoint name.
    pub ckpt_name: Input<String>,
}

/// Struct representing a VAELoader node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VAELoader {
    /// The VAE name.
    pub vae_name: Input<String>,
}

/// Struct representing a VAEDecode node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VAEDecode {
    /// Latent output samples to decode.
    pub samples: NodeConnection,
    /// VAE model input connection.
    pub vae: NodeConnection,
}

/// Struct representing a PreviewImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreviewImage {
    /// The images to preview.
    pub images: NodeConnection,
}

/// Struct representing a KSamplerSelect node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSamplerSelect {
    /// The sampler name.
    pub sampler_name: Input<String>,
}

/// Struct representing a SamplerCustom node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SamplerCustom {
    /// Whether or not to add noise.
    pub add_noise: Input<bool>,
    /// The cfg scale.
    pub cfg: Input<f32>,
    /// The seed.
    pub noise_seed: Input<i64>,
    /// Latent image input connection.
    pub latent_image: NodeConnection,
    /// The model input connection.
    pub model: NodeConnection,
    /// The positive conditioning input connection.
    pub positive: NodeConnection,
    /// The negative conditioning input connection.
    pub negative: NodeConnection,
    /// The sampler input connection.
    pub sampler: NodeConnection,
    /// The sigmas from the scheduler.
    pub sigmas: NodeConnection,
}

/// Struct representing a SDTurboScheduler node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SDTurboScheduler {
    /// The number of steps.
    pub steps: Input<u32>,
    /// The model input connection.
    pub model: NodeConnection,
}

/// Struct representing a ImageOnlyCheckpointLoader node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageOnlyCheckpointLoader {
    /// The checkpoint name.
    pub ckpt_name: Input<String>,
}

/// Struct representing a LoadImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadImage {
    /// UI file selection button.
    #[serde(rename = "choose file to upload")]
    pub file_to_upload: Input<String>,
    /// The name of the image to load.
    pub image: Input<String>,
}

/// Struct representing a SVDimg2vidConditioning node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SVDimg2vidConditioning {
    /// The augmentation level.
    pub augmentation_level: Input<f32>,
    /// The FPS.
    pub fps: Input<u32>,
    /// The video width.
    pub width: Input<u32>,
    /// The video height.
    pub height: Input<u32>,
    /// The motion bucket id.
    pub motion_bucket_id: Input<u32>,
    /// The number of frames.
    pub video_frames: Input<u32>,
    /// The CLIP vision model input connection.
    pub clip_vision: NodeConnection,
    /// The init image input connection.
    pub init_image: NodeConnection,
    /// The VAE model input connection.
    pub vae: NodeConnection,
}

/// Struct representing a VideoLinearCFGGuidance node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoLinearCFGGuidance {
    /// The minimum cfg scale.
    pub min_cfg: Input<f32>,
    /// The model input connection.
    pub model: NodeConnection,
}

/// Struct representing a SaveAnimatedWEBP node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveAnimatedWEBP {
    /// The filename prefix.
    pub filename_prefix: Input<String>,
    /// The FPS.
    pub fps: Input<u32>,
    /// Whether or not to losslessly encode the video.
    pub lossless: Input<bool>,
    /// The encoding method.
    pub method: Input<String>,
    /// The quality.
    pub quality: Input<u32>,
    /// Input images connection.
    pub images: NodeConnection,
}

/// Struct representing a LoraLoader node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoraLoader {
    /// The name of the LORA model.
    pub lora_name: Input<String>,
    /// The model strength.
    pub strength_model: Input<f32>,
    /// The CLIP strength.
    pub strength_clip: Input<f32>,
    /// The model input connection.
    pub model: NodeConnection,
    /// The CLIP input connection.
    pub clip: NodeConnection,
}

/// Struct representing a ModelSamplingDiscrete node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelSamplingDiscrete {
    /// Sampling to use.
    pub sampling: Input<String>,
    /// Use ZSNR.
    pub zsnr: Input<bool>,
    /// The model input connection.
    pub model: NodeConnection,
}

/// Struct representing a SaveImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveImage {
    /// The filename prefix.
    pub filename_prefix: Input<String>,
    /// The image input connection.
    pub images: NodeConnection,
}

/// Struct representing a response to a prompt execution request.
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
    /// The prompt number.
    pub number: u64,
    /// Node errors that have occurred indexed by node id.
    pub node_errors: HashMap<String, serde_json::Value>,
}
