use crate::app::AppContext;
use my_http_server_swagger::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NodeModel {}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueuesModel {
    pub persistence: usize,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct ReaderModel {
    id: String,
    pub name: String,
    pub ip: String,
    pub tables: Vec<String>,
    #[serde(rename = "lastIncomingTime")]
    pub last_incoming_time: String,
    #[serde(rename = "connectedTime")]
    pub connected_time: String,
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StatusModel {
    pub location: LocationModel,
    #[serde(rename = "masterNode")]
    pub master_node: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub queues: QueuesModel,
    pub readers: Vec<ReaderModel>,
    #[serde(rename = "tcpConnections")]
    pub tcp_connections: usize,
}

impl StatusModel {
    pub async fn new(app: &AppContext) -> Self {
        let readers = get_readers(app).await;
        Self {
            location: LocationModel {
                id: app.location.to_string(),
                compress: app.compress_data,
            },
            master_node: None,
            nodes: vec![],
            queues: QueuesModel { persistence: 0 },
            readers: readers.0,
            tcp_connections: readers.1,
        }
    }
}

async fn get_readers(app: &AppContext) -> (Vec<ReaderModel>, usize) {
    let mut result = Vec::new();
    let now = DateTimeAsMicroseconds::now();

    let mut tcp_count = 0;

    for data_reader in app.data_readers.get_all().await {
        match &data_reader.connection {
            crate::data_readers::DataReaderConnection::Tcp(_) => tcp_count += 1,
        }

        let metrics = data_reader.get_metrics().await;

        if let Some(name) = metrics.name {
            result.push(ReaderModel {
                connected_time: metrics.connected.to_rfc3339(),
                last_incoming_time: format!(
                    "{:?}",
                    now.duration_since(metrics.last_incoming_moment)
                ),
                id: metrics.session_id,
                ip: metrics.ip,
                name,
                tables: metrics.tables,
            });
        }
    }

    (result, tcp_count)
}
