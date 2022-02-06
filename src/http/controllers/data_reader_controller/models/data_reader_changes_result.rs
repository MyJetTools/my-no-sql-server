use my_http_server_swagger::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

use crate::http::controllers::row_controller::models::BaseDbRowContract;

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DataReaderChangesResult {
    #[serde(rename = "initTables")]
    pub init_table: Option<Vec<BaseDbRowContract>>,

    #[serde(rename = "initPartitions")]
    pub init_partitions: Option<Vec<BaseDbRowContract>>,

    #[serde(rename = "initRows")]
    pub init_rows: Option<Vec<BaseDbRowContract>>,

    #[serde(rename = "deleteRows")]
    pub delete_rows: Option<Vec<DeleteRowsHttpContract>>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DeleteRowsHttpContract {
    #[serde(rename = "pk")]
    pub partition_key: String,
    #[serde(rename = "rk")]
    pub row_keys: Vec<String>,
}
