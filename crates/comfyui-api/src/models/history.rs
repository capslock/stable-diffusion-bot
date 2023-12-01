use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Image, Prompt};

/// Struct containing task results from the ComfyUI API `history` endpoint.
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct History {
    /// Completed tasks indexed by their uuid.
    pub tasks: HashMap<uuid::Uuid, Task>,
}

/// Struct representing a single task result from the ComfyUI API `history` endpoint.
#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    /// Outputs from the task.
    pub outputs: Outputs,
    /// Information about prompt execution.
    pub prompt: PromptResult,
}

/// Struct representing outputs from a task.
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Outputs {
    /// Outputs from the task indexed by node.
    pub nodes: HashMap<String, NodeOutputOrUnknown>,
}

/// Enumertion of all possible output types from a node.
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NodeOutputOrUnknown {
    /// Enum variant representing image outputs from a node.
    NodeOutput(NodeOutput),
    /// Struct capturing unknown outputs.
    Unknown(serde_json::Value),
}

/// Struct representing image outputs from a node.
#[derive(Serialize, Deserialize, Debug)]
pub struct NodeOutput {
    /// Images from the node.
    pub images: Vec<Image>,
}

/// Struct representing a prompt result.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(from = "(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)")]
#[serde(into = "(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)")]
pub struct PromptResult {
    /// The task number.
    pub num: u64,
    /// The task uuid.
    pub id: uuid::Uuid,
    /// The prompt that was executed.
    pub prompt: Prompt,
    /// Extra data about execution.
    pub extra_data: ExtraData,
    /// Outputs executed for this prompt.
    pub outputs_to_execute: OutputsToExecute,
}

impl From<(u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute)> for PromptResult {
    fn from(
        (num, id, prompt, extra_data, outputs_to_execute): (
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
            extra_data,
            outputs_to_execute,
        }
    }
}

impl From<PromptResult> for (u64, uuid::Uuid, Prompt, ExtraData, OutputsToExecute) {
    fn from(
        PromptResult {
            num,
            id,
            prompt,
            extra_data,
            outputs_to_execute,
        }: PromptResult,
    ) -> Self {
        (num, id, prompt, extra_data, outputs_to_execute)
    }
}

/// Struct representing extra data about prompt execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtraData {
    /// The client id that performed the request.
    pub client_id: uuid::Uuid,
}

/// Struct representing outputs to execute for a prompt.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct OutputsToExecute {
    /// List of nodes which have outputs.
    pub nodes: Vec<String>,
}
