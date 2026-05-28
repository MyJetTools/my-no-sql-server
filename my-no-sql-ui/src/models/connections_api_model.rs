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
pub struct ConnectionWriterApiModel {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub addr: String,
    #[serde(default)]
    pub tables: Vec<String>,
    #[serde(rename = "lastIncomingTime", default)]
    pub last_incoming_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionStatApiModel {
    #[serde(default)]
    pub addr: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(rename = "reqPerSecond", default)]
    pub req_per_second: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConnectionsApiModel {
    #[serde(rename = "incomingPerSecond", default)]
    pub incoming_per_second: usize,
    #[serde(rename = "outgoingPerSecond", default)]
    pub outgoing_per_second: usize,
    #[serde(rename = "writePayloadsPerSecond", default)]
    pub write_payloads_per_second: usize,
    #[serde(rename = "writeBytesPerSecond", default)]
    pub write_bytes_per_second: usize,
    #[serde(default)]
    pub readers: Vec<ConnectionReaderApiModel>,
    #[serde(default)]
    pub writers: Vec<ConnectionWriterApiModel>,
    #[serde(default)]
    pub connections: Vec<ConnectionStatApiModel>,
}
