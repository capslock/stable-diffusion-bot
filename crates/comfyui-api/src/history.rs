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
    pub tasks: HashMap<uuid::Uuid, Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub outputs: Outputs,
    pub prompt: PromptResult,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)")]
#[serde(into = "(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)")]
pub struct PromptResult {
    pub num: u64,
    pub id: uuid::Uuid,
    pub prompt: Prompt,
    pub extra_data: ExtraData,
    pub outputs_to_execute: OutputsToExecute,
}

impl From<(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)> for PromptResult {
    fn from(
        (num, id, prompt, client_info, output_nodes): (
            u64,
            uuid::Uuid,
            Prompt,
            ExtraData,
            OutputsToExecute,
        ),
    ) -> Self {
        Self {
            num,
            id,
            prompt,
            extra_data: client_info,
            outputs_to_execute: output_nodes,
        }
    }
}

impl From<PromptResult> for (u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute) {
    fn from(
        PromptResult {
            num,
            id,
            prompt,
            extra_data: client_info,
            outputs_to_execute: output_nodes,
        }: PromptResult,
    ) -> Self {
        (num, id, prompt, client_info, output_nodes)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraData {
    pub client_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct OutputsToExecute {
    pub nodes: Vec<String>,
}
