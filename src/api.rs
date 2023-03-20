use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Resp {
    pub images: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub info: String,
}
