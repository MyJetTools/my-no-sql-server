use my_http_macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct IsAliveResponse {
    pub name: String,
    pub time: String,
    pub version: String,
    pub env_info: Option<String>,
}
