use serde::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WriterApiModel {
    pub name: String,
    pub version: String,
    pub last_update: String,
    #[serde(default)]
    pub tables: Vec<String>,
}
