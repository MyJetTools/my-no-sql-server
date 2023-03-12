use std::collections::HashMap;

use my_http_server::types::RawDataTyped;
use my_http_server_swagger::*;

use crate::{
    db_sync::DataSynchronizationPeriod,
    http::controllers::row_controller::models::BaseDbRowContract,
};

#[derive(MyHttpInput)]
pub struct BulkDeleteInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: RawDataTyped<HashMap<String, Vec<BaseDbRowContract>>>,
}

#[derive(MyHttpInput)]
pub struct CleanAndBulkInsertInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation";)]
    pub partition_key: Option<String>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(description = "DbRows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: RawDataTyped<HashMap<String, Vec<BaseDbRowContract>>>,
}
