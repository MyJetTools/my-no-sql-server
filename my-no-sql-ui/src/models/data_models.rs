use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableListItemApiModel {
    pub name: String,
    pub persist: bool,
    #[serde(rename = "maxPartitionsAmount", default)]
    pub max_partitions_amount: Option<usize>,
    #[serde(rename = "maxRowsPerPartitionAmount", default)]
    pub max_rows_per_partition_amount: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionsApiModel {
    #[serde(default)]
    pub amount: usize,
    #[serde(default)]
    pub data: Vec<String>,
}
