use my_http_macros::MyHttpInput;
use my_http_macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

use crate::db_sync::DataSynchronizationPeriod;

#[derive(MyHttpInput)]
pub struct RowsCountInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: Option<String>,
}

#[derive(MyHttpInput)]
pub struct InsertOrReplaceInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(description = "DbEntity description"; body_type="BaseDbRowContract")]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct InsertInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "syncPeriod"; description = "Synchronization period"; default="Sec5")]
    pub sync_period: DataSynchronizationPeriod,

    #[http_body(description = "DbEntity description"; body_type = "BaseDbRowContract")]
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct BaseDbRowContract {
    #[serde(rename = "partitionKey")]
    pub partition_key: String,

    #[serde(rename = "rowKey")]
    pub row_key: String,

    #[serde(rename = "timeStamp")]
    pub time_stamp: String,

    pub expires: Option<String>,
}
