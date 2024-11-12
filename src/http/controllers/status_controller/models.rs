use crate::app::AppContext;
use my_http_server::macros::*;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use super::{non_initialized::NonInitializedModel, status_bar_model::StatusBarModel};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TableModel {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
    #[serde(rename = "maxRowsPerPartition")]
    pub max_rows_per_partition: Option<usize>,
    #[serde(rename = "partitionsCount")]
    pub partitions_count: usize,
    #[serde(rename = "dataSize")]
    pub data_size: usize,
    #[serde(rename = "recordsAmount")]
    pub records_amount: usize,
    #[serde(rename = "expirationIndex")]
    pub expiration_index_records_amount: usize,
    #[serde(rename = "lastUpdateTime")]
    pub last_update_time: i64,
    #[serde(rename = "lastPersistTime")]
    pub last_persist_time: Option<i64>,
    #[serde(rename = "lastPersistDuration")]
    pub last_persist_duration: Vec<usize>,
    #[serde(rename = "nextPersistTime")]
    pub next_persist_time: Option<i64>,
    #[serde(rename = "persistAmount")]
    pub persist_amount: usize,
    #[serde(rename = "avgEntitySize")]
    pub avg_entity_size: usize,
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
    #[serde(rename = "pendingToSend")]
    pub pending_to_send: usize,
    #[serde(rename = "sentPerSecond")]
    pub sent_per_second: Vec<usize>,
    #[serde(rename = "isNode")]
    pub is_node: bool,
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct StatusModel {
    #[serde(rename = "notInitialized", skip_serializing_if = "Option::is_none")]
    not_initialized: Option<NonInitializedModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initialized: Option<InitializedModel>,
    #[serde(rename = "statusBar")]
    status_bar: StatusBarModel,
}

impl StatusModel {
    pub async fn new(app: &AppContext) -> Self {
        let (readers, tcp, http) = get_readers(app).await;
        let tables = app.db.get_tables().await;

        let mut tables_model = Vec::new();

        for table in &tables {
            let attr = table.get_attributes().await;
            let metrics = crate::operations::get_table_metrics(app, table.as_ref()).await;

            let last_persist_time = if let Some(last_persist_time) = metrics.last_persist_time {
                Some(last_persist_time.unix_microseconds)
            } else {
                None
            };

            let table_model = TableModel {
                name: table.name.clone(),
                avg_entity_size: metrics.avg_entity_size,
                persist: attr.persist,
                max_partitions_amount: attr.max_partitions_amount,
                max_rows_per_partition: attr.max_rows_per_partition_amount,
                partitions_count: metrics.partitions_amount,
                data_size: metrics.table_size,
                expiration_index_records_amount: metrics.expiration_index_records_amount,
                records_amount: metrics.records_amount,
                last_update_time: metrics.last_update_time.unix_microseconds,
                last_persist_time,
                persist_amount: metrics.persist_amount,
                last_persist_duration: metrics.last_persist_duration,
                next_persist_time: if let Some(next_persist_time) = metrics.next_persist_time {
                    Some(next_persist_time.unix_microseconds)
                } else {
                    None
                },
            };

            tables_model.push(table_model);
        }

        let used_http_connections = app.metrics.get_http_connections_amount();

        let writers = WriterApiModel::new(app).await;

        if app.states.is_initialized() {
            return Self {
                not_initialized: None,
                initialized: Some(InitializedModel::new(readers, tables_model, writers)),
                status_bar: StatusBarModel::new(
                    app,
                    tcp,
                    http,
                    tables.len(),
                    used_http_connections,
                ),
            };
        }

        return Self {
            not_initialized: Some(NonInitializedModel::new(app).await),
            initialized: None,
            status_bar: StatusBarModel::new(app, tcp, http, tables.len(), used_http_connections),
        };
    }
}
#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct InitializedModel {
    pub readers: Vec<ReaderModel>,
    pub writers: Vec<WriterApiModel>,
    pub tables: Vec<TableModel>,
}

impl InitializedModel {
    pub fn new(
        readers: Vec<ReaderModel>,
        tables: Vec<TableModel>,
        writers: Vec<WriterApiModel>,
    ) -> Self {
        Self {
            readers,
            tables,
            writers,
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

        result.push(ReaderModel {
            connected_time: metrics.connected.to_rfc3339(),
            last_incoming_time: format!(
                "{:?}",
                now.duration_since(metrics.last_incoming_moment)
                    .as_positive_or_zero()
            ),
            id: metrics.session_id,
            ip: metrics.ip,
            name: metrics.name,
            tables: metrics.tables,
            pending_to_send: metrics.pending_to_send,
            sent_per_second: data_reader.get_sent_per_second().await,
            is_node: data_reader.is_node(),
        });
    }

    (result, tcp_count, http_count)
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct WriterApiModel {
    pub name: String,
    pub version: String,
    pub last_update: String,
}

impl WriterApiModel {
    pub async fn new(app: &AppContext) -> Vec<Self> {
        app.http_writers
            .get(|name, itm| Self {
                name: name.to_string(),
                version: itm.version.to_string(),
                last_update: itm.last_ping.to_rfc3339(),
            })
            .await
    }
}
