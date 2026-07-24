use std::collections::HashMap;

use my_http_server::macros::*;
use my_http_server::RawDataTyped;
use serde::{Deserialize, Serialize};

use crate::{
    db_sync::DataSynchronizationPeriod,
    http_server::controllers::row_controller::models::BaseDbRowContract,
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
pub struct CleanAndBulkInsertByChunksInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation. Taken into account only when a new process is started";)]
    pub partition_key: Option<String>,

    #[http_query(name = "processId"; description = "Id of the process. Omit it to start a new process - the id is returned in the response";)]
    pub process_id: Option<String>,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,

    #[http_body_raw(description = "DbRows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}

#[derive(MyHttpInput)]
pub struct BulkProcessInputContract {
    #[http_query(name = "processId"; description = "Id of the process")]
    pub process_id: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,
}

#[derive(MyHttpInput)]
pub struct CancelBulkProcessInputContract {
    #[http_query(name = "processId"; description = "Id of the process")]
    pub process_id: String,

    #[http_header(name = "session", description = "Writer session id")]
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BulkProcessResponse {
    #[serde(rename = "processId")]
    pub process_id: String,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body_raw(description = "Rows")]
    pub body: RawDataTyped<Vec<BaseDbRowContract>>,
}
