use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Image, Prompt};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum HistoryOrUnknown {
    History(History),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct History {
    pub prompts: HashMap<uuid::Uuid, Info>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    pub outputs: Outputs,
    pub prompt: PromptData,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Outputs {
    pub nodes: HashMap<String, NodeOutputOrUnknown>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeOutputOrUnknown {
    NodeOutput(NodeOutput),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeOutput {
    pub images: Vec<Image>,
}

//#[derive(Serialize, Deserialize, Debug)]
//pub struct PromptData(u64, uuid::Uuid, Prompt, ClientInfo, OutputNodes);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(u64, uuid::Uuid, Prompt, ClientInfo, OutputNodes)")]
#[serde(into = "(u64, uuid::Uuid, Prompt, ClientInfo, OutputNodes)")]
pub struct PromptData {
    pub num: u64,
    pub id: uuid::Uuid,
    pub prompt: Prompt,
    pub client_info: ClientInfo,
    pub output_nodes: OutputNodes,
}

impl From<(u64, uuid::Uuid, Prompt, ClientInfo, OutputNodes)> for PromptData {
    fn from(
        (num, id, prompt, client_info, output_nodes): (
            u64,
            uuid::Uuid,
            Prompt,
            ClientInfo,
            OutputNodes,
        ),
    ) -> Self {
        Self {
            num,
            id,
            prompt,
            client_info,
            output_nodes,
        }
    }
}

impl From<PromptData> for (u64, uuid::Uuid, Prompt, ClientInfo, OutputNodes) {
    fn from(
        PromptData {
            num,
            id,
            prompt,
            client_info,
            output_nodes,
        }: PromptData,
    ) -> Self {
        (num, id, prompt, client_info, output_nodes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientInfo {
    pub client_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct OutputNodes {
    pub nodes: Vec<String>,
}
