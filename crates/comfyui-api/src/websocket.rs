use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PreviewOrUpdate {
    Preview(Preview),
    Update(UpdateOrUnknown),
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Preview(pub Vec<u8>);

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdateOrUnknown {
    Update(Update),
    Unknown(serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Update {
    Progress(ProgressData),
    Status { status: StatusData },
    ExecutionStart(ExecutionStartData),
    ExecutionCached(ExecutionCachedData),
    Executing(ExecutingData),
    Executed(ExecutedData),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgressData {
    pub value: u64,
    pub max: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusData {
    pub exec_info: ExecInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecInfo {
    pub queue_remaining: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionStartData {
    prompt_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionCachedData {
    prompt_id: String,
    nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutingData {
    pub prompt_id: String,
    pub node: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutedData {
    pub prompt_id: String,
    pub node: String,
    pub output: Output,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub images: Vec<Image>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub filename: String,
    pub subfolder: String,
    #[serde(rename = "type")]
    pub folder_type: String,
}
