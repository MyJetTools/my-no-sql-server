use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiModel {
    pub name: String,
    pub time: String,
    pub version: String,
    pub env_info: Option<String>,
}
