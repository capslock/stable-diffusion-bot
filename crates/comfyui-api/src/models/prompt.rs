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
    Unknown(serde_json::Value),
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
    // TODO: Implement other node types.
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

/// Struct representing a KSampler node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSampler {
    /// The cfg scale parameter.
    pub cfg: f32,
    /// The denoise parameter.
    pub denoise: f32,
    /// The sampler name.
    pub sampler_name: String,
    /// The scheduler used.
    pub scheduler: String,
    /// The seed.
    pub seed: i64,
    /// The number of steps.
    pub steps: i32,
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
    pub text: String,
    /// The CLIP model input connection.
    pub clip: NodeConnection,
}

/// Struct representing an EmptyLatentImage node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyLatentImage {
    /// The batch size.
    pub batch_size: u32,
    /// The image width.
    pub width: u32,
    /// The image height.
    pub height: u32,
}

/// Struct representing a CheckpointLoaderSimple node.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointLoaderSimple {
    /// The checkpoint name.
    pub ckpt_name: String,
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
