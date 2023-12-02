use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// An enum representing a websocket message.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PreviewOrUpdate {
    /// Enum variant representing an image preview.
    Preview(Preview),
    /// Enum variant representing an update.
    Update(Update),
}

/// Struct representing an image preview.
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Preview(pub Vec<u8>);

/// Enum of possible update variants.
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Enum variant representing a status update.
    Status { status: Status },
    /// Enum variant representing a progress update.
    Progress(Progress),
    /// Enum variant representing an execution start update.
    ExecutionStart(ExecutionStart),
    /// Enum variant representing an executing update.
    Executing(Executing),
    /// Enum variant representing an executed update.
    Executed(Executed),
    /// Enum variant representing an execution cached update.
    ExecutionCached(ExecutionCached),
    /// Enum variant representing an execution interrupted update.
    ExecutionInterrupted(ExecutionInterrupted),
    /// Enum variant representing an execution error update.
    ExecutionError(ExecutionError),
}

/// Struct representing a progress update.
#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    /// The current progress value.
    pub value: u64,
    /// The maximum progress value.
    pub max: u64,
}

/// Struct representing a status update.
#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    /// The current status.
    pub exec_info: ExecInfo,
}

/// Struct representing execution information.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecInfo {
    /// Number of items remaining in the queue.
    pub queue_remaining: u64,
}

/// Struct representing an execution start update.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionStart {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
}

/// Struct representing an execution cached update.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionCached {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
    /// The ids of the nodes that were cached.
    pub nodes: Vec<String>,
}

/// Struct representing an executing update.
#[derive(Serialize, Deserialize, Debug)]
pub struct Executing {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
    /// The node that is executing.
    pub node: Option<String>,
}

/// Struct representing an executed update.
#[derive(Serialize, Deserialize, Debug)]
pub struct Executed {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
    /// The node that was executed.
    pub node: String,
    /// The output of the node.
    pub output: Output,
}

/// Struct representing an output.
#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    /// A list of images.
    pub images: Vec<Image>,
}

/// Struct representing an image.
#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    /// The filename of the image.
    pub filename: String,
    /// The subfolder.
    pub subfolder: String,
    /// The folder type.
    #[serde(rename = "type")]
    pub folder_type: String,
}

/// Struct representing an execution interrupted update.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionInterrupted {
    /// The prompt id.
    pub prompt_id: uuid::Uuid,
    /// The node that was executing.
    pub node_id: String,
    /// The type of the node.
    pub node_type: String,
    /// What was executed prior to interruption.
    pub executed: Vec<String>,
}

/// Struct representing an execution error update.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionError {
    /// The state of execution that was interrupted.
    #[serde(flatten)]
    pub execution_status: ExecutionInterrupted,
    /// The exception message.
    pub exception_message: String,
    /// The exception type.
    pub exception_type: String,
    /// The traceback.
    pub traceback: Vec<String>,
    /// The current inputs.
    pub current_inputs: CurrentInputs,
    /// The current outputs.
    pub current_outputs: CurrentOutputs,
}

/// Struct representing the current inputs when the execution error occurred.
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct CurrentInputs {
    /// Hashmap of inputs keyed by input name.
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Struct representing the current outputs when the execution error occurred.
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct CurrentOutputs {
    /// Hashmap of outputs keyed by node id.
    pub outputs: HashMap<String, Vec<serde_json::Value>>,
}
