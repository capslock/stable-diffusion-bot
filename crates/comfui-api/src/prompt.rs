use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Prompt {
    #[serde(flatten)]
    pub workflow: HashMap<String, NodeOrUnknown>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeOrUnknown {
    Node(Node),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "class_type", content = "inputs")]
pub enum Node {
    KSampler(KSamplerInputs),
    CLIPTextEncode(CLIPTextEncodeInputs),
    EmptyLatentImage(EmptyLatentImageInputs),
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CLIPTextEncodeInputs {
    pub text: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyLatentImageInputs {
    pub batch_size: u32,
    pub width: u32,
    pub height: u32,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
