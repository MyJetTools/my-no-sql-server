use crate::db_sync::DataSynchronizationPeriod;
use my_http_macros::MyHttpInput;

#[derive(MyHttpInput)]
pub struct BulkDeleteInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct CleanAndBulkInsertContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key to clean before bulk insert operation";)]
    pub partition_key: Option<String>,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct BulkInsertOrReplaceContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(
        description = "PartitionToDelete1:[RowToDelete1, RowToDelete2, RowToDelete3],[PartitionToDelete1]:[RowToDelete1, RowToDelete2, RowToDelete3]"
    )]
    pub body: Vec<u8>,
}
