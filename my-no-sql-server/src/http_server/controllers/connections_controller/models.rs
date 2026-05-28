use crate::app::AppContext;
use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct ConnectionReaderModel {
    pub id: String,
    pub name: String,
    pub ip: String,
    #[serde(rename = "incomingPerSecond")]
    pub incoming_per_second: usize,
    #[serde(rename = "outgoingPerSecond")]
    pub outgoing_per_second: usize,
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: usize,
    #[serde(rename = "isNode")]
    pub is_node: bool,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct ConnectionsModel {
    #[serde(rename = "incomingPerSecond")]
    pub incoming_per_second: usize,
    #[serde(rename = "outgoingPerSecond")]
    pub outgoing_per_second: usize,
    pub readers: Vec<ConnectionReaderModel>,
}

impl ConnectionsModel {
    pub async fn new(app: &AppContext) -> Self {
        let mut readers = Vec::new();
        let mut total_incoming = 0;
        let mut total_outgoing = 0;

        for data_reader in app.data_readers.get_all().await {
            let (incoming, outgoing) = data_reader.get_traffic_per_second();
            total_incoming += incoming;
            total_outgoing += outgoing;

            let metrics = data_reader.get_metrics().await;

            readers.push(ConnectionReaderModel {
                id: metrics.session_id,
                name: metrics.name,
                ip: metrics.ip,
                incoming_per_second: incoming,
                outgoing_per_second: outgoing,
                pending_to_send: metrics.pending_to_send,
                is_node: data_reader.is_node(),
            });
        }

        Self {
            incoming_per_second: total_incoming,
            outgoing_per_second: total_outgoing,
            readers,
        }
    }
}
