use serde::*;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct LocationApiModel {
    pub id: String,
    #[serde(default)]
    pub compress: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct StatusBarApiModel {
    #[serde(default)]
    pub location: LocationApiModel,
    #[serde(rename = "persistAmount", default)]
    pub persist_amount: usize,
    #[serde(rename = "tcpConnections", default)]
    pub tcp_connections: usize,
    #[serde(rename = "tablesAmount", default)]
    pub tables_amount: usize,
    #[serde(rename = "httpConnections", default)]
    pub http_connections: usize,
    #[serde(rename = "masterNode", default)]
    pub master_node: Option<String>,
    #[serde(rename = "usedHttpConnections", default)]
    pub used_http_connections: i64,
    #[serde(rename = "syncQueueSize", default)]
    pub sync_queue_size: i64,
}
