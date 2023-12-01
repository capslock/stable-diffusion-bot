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
    Progress(Progress),
    Status { status: Status },
    ExecutionStart(ExecutionStart),
    ExecutionCached(ExecutionCached),
    Executing(Executing),
    Executed(Executed),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Progress {
    pub value: u64,
    pub max: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub exec_info: ExecInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecInfo {
    pub queue_remaining: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionStart {
    pub prompt_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExecutionCached {
    pub prompt_id: uuid::Uuid,
    pub nodes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Executing {
    pub prompt_id: uuid::Uuid,
    pub node: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Executed {
    pub prompt_id: uuid::Uuid,
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
