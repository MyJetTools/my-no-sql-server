use serde::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TableApiModel {
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
    #[serde(rename = "lastPersistDuration", default)]
    pub last_persist_duration: Vec<usize>,
    #[serde(rename = "nextPersistTime")]
    pub next_persist_time: Option<i64>,
    #[serde(rename = "persistAmount")]
    pub persist_amount: usize,
    #[serde(rename = "avgEntitySize")]
    pub avg_entity_size: usize,
}
