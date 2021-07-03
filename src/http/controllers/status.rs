use std::sync::Arc;

use crate::{
    app::AppServices,
    db::{FailOperationResult, OperationResult},
};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
struct NodeModel {}

#[derive(Serialize, Deserialize, Debug)]
struct LocationModel {
    pub id: String,
    pub compress: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct QueuesModel {
    pub persistence: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReaderModel {
    pub name: String,
    pub ip: String,
    pub tables: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct StatusModel {
    pub location: LocationModel,
    #[serde(rename = "masterNode")]
    pub master_node: Option<String>,
    pub nodes: Vec<NodeModel>,
    pub queues: QueuesModel,
    pub readers: Vec<ReaderModel>,
}

async fn get_readers(app: &AppServices) -> Vec<ReaderModel> {
    let mut result = Vec::new();

    for data_reader in app.data_readers.get_all().await {
        let read_data = data_reader.data.read().await;
        result.push(ReaderModel {
            ip: read_data.ip.clone(),
            name: read_data.to_string(),
            tables: read_data
                .tables
                .keys()
                .into_iter()
                .map(|name| name.to_string())
                .collect(),
        });
    }

    result
}

pub async fn get(app: Arc<AppServices>) -> Result<OperationResult, FailOperationResult> {
    let model = StatusModel {
        location: LocationModel {
            id: app.settings.location.to_string(),
            compress: app.settings.compress_data,
        },
        master_node: None,
        nodes: vec![],
        queues: QueuesModel { persistence: 0 },
        readers: get_readers(app.as_ref()).await,
    };

    let json = serde_json::to_string(&model).unwrap();

    return Ok(OperationResult::OkWithJsonString { json });
}
