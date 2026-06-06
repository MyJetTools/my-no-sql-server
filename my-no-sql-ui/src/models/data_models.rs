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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PartitionMetricApiModel {
    #[serde(rename = "partitionKey")]
    pub partition_key: String,
    #[serde(rename = "recordsCount", default)]
    pub records_count: usize,
    #[serde(rename = "dataSize", default)]
    pub data_size: usize,
}
