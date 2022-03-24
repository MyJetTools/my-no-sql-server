use my_http_server_swagger::*;
use serde::{Deserialize, Serialize};
#[derive(MyHttpInput)]
pub struct GetHighestRowsAndBelowInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_query(name = "rowKey"; description = "Row Key")]
    pub row_key: String,

    #[http_query(name = "maxAmount"; description = "Limit amount of records we are going to get")]
    pub max_amount: usize,
}

#[derive(MyHttpInput)]
pub struct GetSinglePartitionMultipleRowsActionInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: String,

    #[http_body(description = "Row keys")]
    pub body: Vec<u8>,
}

#[derive(MyHttpInput)]
pub struct DeletePartitionsInputContract {
    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_body(name = "partitionKeys"; description = "Partition Keys to delete", required = true, body_type="DeletePartitionsModel" )]
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeletePartitionsModel {
    #[serde(rename = "partitionKeys")]
    pub partition_keys: Vec<u8>,
}
