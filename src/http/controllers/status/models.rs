use crate::app::AppContext;
use my_http_server_swagger::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use super::non_initialized::NonInitializedModel;

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct NodeModel {}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct LocationModel {
    pub id: String,
    pub compress: bool,
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
    #[serde(rename = "notInitialized")]
    not_initialized: Option<NonInitializedModel>,
    initialized: Option<InitializedModel>,
}

impl StatusModel {
    pub async fn new(app: &AppContext) -> Self {
        if app.states.is_initialized() {
            return Self {
                not_initialized: None,
                initialized: Some(InitializedModel::new(app).await),
            };
        }

        return Self {
            not_initialized: Some(NonInitializedModel::new(app).await),
            initialized: None,
        };
    }
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct InitializedModel {
    pub location: LocationModel,
    #[serde(rename = "masterNode")]
    pub master_node: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub readers: Vec<ReaderModel>,
    #[serde(rename = "tcpConnections")]
    pub tcp_connections: usize,
    #[serde(rename = "tablesAmount")]
    pub tables_amount: usize,
    #[serde(rename = "httpConnections")]
    pub http_connections: usize,
}

impl InitializedModel {
    pub async fn new(app: &AppContext) -> Self {
        let readers = get_readers(app).await;
        let tables_amount = app.db.get_tables_amount().await;

        Self {
            location: LocationModel {
                id: app.location.to_string(),
                compress: app.compress_data,
            },
            master_node: None,
            nodes: vec![],
            readers: readers.0,
            tcp_connections: readers.1,
            http_connections: readers.2,
            tables_amount,
        }
    }
}

async fn get_readers(app: &AppContext) -> (Vec<ReaderModel>, usize, usize) {
    let mut result = Vec::new();
    let now = DateTimeAsMicroseconds::now();

    let mut tcp_count = 0;
    let mut http_count = 0;

    for data_reader in app.data_readers.get_all().await {
        match &data_reader.connection {
            crate::data_readers::DataReaderConnection::Tcp(_) => tcp_count += 1,
            crate::data_readers::DataReaderConnection::Http(_) => http_count += 1,
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

    (result, tcp_count, http_count)
}
