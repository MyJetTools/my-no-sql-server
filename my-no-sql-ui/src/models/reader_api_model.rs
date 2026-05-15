use serde::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ReaderApiModel {
    pub id: String,
    pub name: String,
    pub ip: String,
    #[serde(default)]
    pub tables: Vec<String>,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    #[serde(rename = "connectedTime")]
    pub connected_time: String,
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: usize,
    #[serde(rename = "sentPerSecond", default)]
    pub sent_per_second: Vec<usize>,
    #[serde(rename = "isNode", default)]
    pub is_node: bool,
}
