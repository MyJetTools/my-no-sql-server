use my_http_server_swagger::*;

use crate::db_sync::DataSynchronizationPeriod;

#[derive(MyHttpInput)]
pub struct BulkDeleteInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct CleanAndBulkInsertInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation";)]
    pub partition_key: Option<String>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default)]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}
