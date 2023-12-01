use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Prompt {
    #[serde(flatten)]
    pub workflow: HashMap<String, NodeOrUnknown>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NodeOrUnknown {
    Node(Node),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "class_type", content = "inputs")]
pub enum Node {
    KSampler(KSampler),
    CLIPTextEncode(CLIPTextEncode),
    EmptyLatentImage(EmptyLatentImage),
    CheckpointLoaderSimple(CheckpointLoaderSimple),
    // TODO: Implement other node types.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(String, u32)")]
#[serde(into = "(String, u32)")]
pub struct NodeConnection {
    pub node_id: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSampler {
    pub cfg: f32,
    pub denoise: f32,
    pub sampler_name: String,
    pub scheduler: String,
    pub seed: i64,
    pub steps: i32,
    pub positive: NodeConnection,
    pub negative: NodeConnection,
    pub model: NodeConnection,
    pub latent_image: NodeConnection,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CLIPTextEncode {
    pub text: String,
    pub clip: NodeConnection,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyLatentImage {
    pub batch_size: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointLoaderSimple {
    pub ckpt_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub prompt_id: uuid::Uuid,
    pub number: u64,
    pub node_errors: HashMap<String, serde_json::Value>,
}
