use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Prompt {
    #[serde(flatten)]
    pub workflow: HashMap<String, NodeOrUnknown>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NodeOrUnknown {
    Node(Node),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "class_type", content = "inputs")]
pub enum Node {
    KSampler(KSamplerInputs),
    CLIPTextEncode(CLIPTextEncodeInputs),
    EmptyLatentImage(EmptyLatentImageInputs),
    CheckpointLoaderSimple(CheckpointLoaderSimpleInputs),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KSamplerInputs {
    pub cfg: f32,
    pub denoise: f32,
    pub sampler_name: String,
    pub scheduler: String,
    pub seed: i64,
    pub steps: i32,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CLIPTextEncodeInputs {
    pub text: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmptyLatentImageInputs {
    pub batch_size: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckpointLoaderSimpleInputs {
    pub ckpt_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub prompt_id: uuid::Uuid,
    pub number: u64,
    pub node_errors: HashMap<String, serde_json::Value>,
}
