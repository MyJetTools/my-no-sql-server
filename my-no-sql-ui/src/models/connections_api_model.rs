use serde::*;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionReaderApiModel {
    pub id: String,
    pub name: String,
    pub ip: String,
    #[serde(rename = "incomingPerSecond", default)]
    pub incoming_per_second: usize,
    #[serde(rename = "outgoingPerSecond", default)]
    pub outgoing_per_second: usize,
    #[serde(rename = "pendingToSend", default)]
    pub pending_to_send: usize,
    #[serde(rename = "isNode", default)]
    pub is_node: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionsApiModel {
    #[serde(rename = "incomingPerSecond", default)]
    pub incoming_per_second: usize,
    #[serde(rename = "outgoingPerSecond", default)]
    pub outgoing_per_second: usize,
    #[serde(default)]
    pub readers: Vec<ConnectionReaderApiModel>,
}
