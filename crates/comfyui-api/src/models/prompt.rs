use std::{any::Any, collections::HashMap};

use serde::{Deserialize, Serialize};

/// Struct representing a prompt workflow.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Prompt {
    /// The prompt workflow, indexed by node id.
    #[serde(flatten)]
    pub workflow: HashMap<String, NodeOrUnknown>,
}

impl Prompt {
    pub fn get_node_by_id(&self, id: &str) -> Option<&dyn Node> {
        match self.workflow.get(id) {
            Some(NodeOrUnknown::Node(node)) => Some(node.as_ref()),
            Some(NodeOrUnknown::GenericNode(node)) => Some(node),
            _ => None,
        }
    }
}

/// Enum capturing all possible node types.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeOrUnknown {
    /// Enum variant representing a known node.
    Node(Box<dyn Node>),
    /// Variant capturing unknown nodes.
    GenericNode(GenericNode),
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait to allow downcasting to `dyn Any`.
pub trait AsAny {
    /// Get a reference to `dyn Any`.
    fn as_any(&self) -> &dyn Any;
}

/// Get a reference to a node of a specific type.
///
/// # Arguments
///
/// * `node` - The node to get a reference to.
///
/// # Returns
///
/// A reference to the node of the specified type if the node is of the specified type, otherwise `None`.
pub fn as_node<T: Node + 'static>(node: &dyn Node) -> Option<&T> {
    node.as_any().downcast_ref::<T>()
}

#[typetag::serde(tag = "class_type", content = "inputs")]
pub trait Node: std::fmt::Debug + AsAny {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_>;
}

/// Struct representing a generic node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericNode {
    /// The node class type.
    pub class_type: String,
    /// The node inputs.
    pub inputs: HashMap<String, GenericValue>,
}

#[typetag::serde]
impl Node for GenericNode {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.inputs.values().filter_map(|input| input.node_id()))
    }
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

impl GenericValue {
    /// Get the node id of the input.
    pub fn node_id(&self) -> Option<&str> {
        match self {
            GenericValue::NodeConnection(node_connection) => Some(&node_connection.node_id),
            _ => None,
        }
    }
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

impl<T> Input<T> {
    /// Get the value of the input.
    pub fn value(&self) -> Option<&T> {
        match self {
            Input::NodeConnection(_) => None,
            Input::Value(value) => Some(value),
        }
    }

    /// Get the node connection of the input.
    pub fn node_connection(&self) -> Option<&NodeConnection> {
        match self {
            Input::NodeConnection(node_connection) => Some(node_connection),
            Input::Value(_) => None,
        }
    }

    /// Get the node id of the input.
    pub fn node_id(&self) -> Option<&str> {
        self.node_connection()
            .map(|node_connection| node_connection.node_id.as_str())
    }
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

#[typetag::serde]
impl Node for KSampler {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [
            self.cfg.node_id(),
            self.denoise.node_id(),
            self.sampler_name.node_id(),
            self.scheduler.node_id(),
            self.seed.node_id(),
            self.steps.node_id(),
        ]
        .into_iter()
        .flatten();
        Box::new(inputs.chain([
            self.positive.node_id.as_str(),
            self.negative.node_id.as_str(),
            self.model.node_id.as_str(),
            self.latent_image.node_id.as_str(),
        ]))
    }
}

/// Struct representing a CLIPTextEncode node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CLIPTextEncode {
    /// The text to encode.
    pub text: Input<String>,
    /// The CLIP model input connection.
    pub clip: NodeConnection,
}

#[typetag::serde]
impl Node for CLIPTextEncode {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [self.text.node_id(), Some(self.clip.node_id.as_str())]
                .into_iter()
                .flatten(),
        )
    }
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

#[typetag::serde]
impl Node for EmptyLatentImage {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [
                self.batch_size.node_id(),
                self.width.node_id(),
                self.height.node_id(),
            ]
            .into_iter()
            .flatten(),
        )
    }
}

/// Struct representing a CheckpointLoaderSimple node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointLoaderSimple {
    /// The checkpoint name.
    pub ckpt_name: Input<String>,
}

#[typetag::serde]
impl Node for CheckpointLoaderSimple {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.ckpt_name.node_id()].into_iter().flatten())
    }
}

/// Struct representing a VAELoader node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VAELoader {
    /// The VAE name.
    pub vae_name: Input<String>,
}

#[typetag::serde]
impl Node for VAELoader {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.vae_name.node_id()].into_iter().flatten())
    }
}

/// Struct representing a VAEDecode node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VAEDecode {
    /// Latent output samples to decode.
    pub samples: NodeConnection,
    /// VAE model input connection.
    pub vae: NodeConnection,
}

#[typetag::serde]
impl Node for VAEDecode {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.samples.node_id.as_str(), self.vae.node_id.as_str()].into_iter())
    }
}

/// Struct representing a PreviewImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PreviewImage {
    /// The images to preview.
    pub images: NodeConnection,
}

#[typetag::serde]
impl Node for PreviewImage {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.images.node_id.as_str()].into_iter())
    }
}

/// Struct representing a KSamplerSelect node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSamplerSelect {
    /// The sampler name.
    pub sampler_name: Input<String>,
}

#[typetag::serde]
impl Node for KSamplerSelect {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.sampler_name.node_id()].into_iter().flatten())
    }
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

#[typetag::serde]
impl Node for SamplerCustom {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [
            self.add_noise.node_id(),
            self.cfg.node_id(),
            self.noise_seed.node_id(),
        ]
        .into_iter()
        .flatten();
        Box::new(inputs.chain([
            self.latent_image.node_id.as_str(),
            self.model.node_id.as_str(),
            self.positive.node_id.as_str(),
            self.negative.node_id.as_str(),
            self.sampler.node_id.as_str(),
            self.sigmas.node_id.as_str(),
        ]))
    }
}

/// Struct representing a SDTurboScheduler node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SDTurboScheduler {
    /// The number of steps.
    pub steps: Input<u32>,
    /// The model input connection.
    pub model: NodeConnection,
}

#[typetag::serde]
impl Node for SDTurboScheduler {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [self.steps.node_id(), Some(self.model.node_id.as_str())]
                .into_iter()
                .flatten(),
        )
    }
}

/// Struct representing a ImageOnlyCheckpointLoader node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageOnlyCheckpointLoader {
    /// The checkpoint name.
    pub ckpt_name: Input<String>,
}

#[typetag::serde]
impl Node for ImageOnlyCheckpointLoader {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new([self.ckpt_name.node_id()].into_iter().flatten())
    }
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

#[typetag::serde]
impl Node for LoadImage {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [self.file_to_upload.node_id(), self.image.node_id()]
                .into_iter()
                .flatten(),
        )
    }
}

/// Struct representing a SVDimg2vidConditioning node.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename = "SVD_img2vid_Conditioning")]
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

#[typetag::serde]
impl Node for SVDimg2vidConditioning {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [
            self.augmentation_level.node_id(),
            self.fps.node_id(),
            self.width.node_id(),
            self.height.node_id(),
            self.motion_bucket_id.node_id(),
            self.video_frames.node_id(),
        ]
        .into_iter()
        .flatten();
        Box::new(inputs.chain([
            self.clip_vision.node_id.as_str(),
            self.init_image.node_id.as_str(),
            self.vae.node_id.as_str(),
        ]))
    }
}

/// Struct representing a VideoLinearCFGGuidance node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VideoLinearCFGGuidance {
    /// The minimum cfg scale.
    pub min_cfg: Input<f32>,
    /// The model input connection.
    pub model: NodeConnection,
}

#[typetag::serde]
impl Node for VideoLinearCFGGuidance {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [self.min_cfg.node_id(), Some(self.model.node_id.as_str())]
                .into_iter()
                .flatten(),
        )
    }
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

#[typetag::serde]
impl Node for SaveAnimatedWEBP {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [
            self.filename_prefix.node_id(),
            self.fps.node_id(),
            self.lossless.node_id(),
            self.method.node_id(),
            self.quality.node_id(),
        ]
        .into_iter()
        .flatten();
        Box::new(inputs.chain([self.images.node_id.as_str()]))
    }
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

#[typetag::serde]
impl Node for LoraLoader {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [
            self.lora_name.node_id(),
            self.strength_model.node_id(),
            self.strength_clip.node_id(),
        ]
        .into_iter()
        .flatten();
        Box::new(inputs.chain([self.model.node_id.as_str(), self.clip.node_id.as_str()]))
    }
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

#[typetag::serde]
impl Node for ModelSamplingDiscrete {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        let inputs = [self.sampling.node_id(), self.zsnr.node_id()]
            .into_iter()
            .flatten();
        Box::new(inputs.chain([self.model.node_id.as_str()]))
    }
}

/// Struct representing a SaveImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveImage {
    /// The filename prefix.
    pub filename_prefix: Input<String>,
    /// The image input connection.
    pub images: NodeConnection,
}

#[typetag::serde]
impl Node for SaveImage {
    fn connections(&'_ self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(
            [
                self.filename_prefix.node_id(),
                Some(self.images.node_id.as_str()),
            ]
            .into_iter()
            .flatten(),
        )
    }
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
