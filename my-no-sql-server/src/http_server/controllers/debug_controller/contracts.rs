use my_http_server::macros::*;
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct GetRowStatisticsInputData {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,
    #[http_query(name = "partitionKey"; description = "PartitionKey ")]
    pub partition_key: String,
    #[http_query(name = "rowKey"; description = "RowKey")]
    pub row_key: String,
}
#[derive(Deserialize, Serialize, MyHttpObjectStructure)]
pub struct RowStatisticsContract {
    pub partition_read_time: String,
    pub partition_write_time: String,
    pub row_read_time: String,
    pub row_write_time: String,
}
