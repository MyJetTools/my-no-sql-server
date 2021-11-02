use serde::{Deserialize, Serialize};

pub const METADATA_BLOB_NAME: &str = ".metadata";

#[derive(Serialize, Deserialize, Debug)]
pub struct TableMetadataFileContract {
    #[serde(rename = "Persist")]
    #[serde(default = "default_persist")]
    pub persist: bool,
    #[serde(rename = "MaxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

impl TableMetadataFileContract {
    pub fn parse(content: &[u8]) -> Self {
        let parse_result = serde_json::from_slice::<TableMetadataFileContract>(content);

        match parse_result {
            Ok(res) => res,
            Err(_) => TableMetadataFileContract {
                max_partitions_amount: None,
                persist: true,
            },
        }
    }
}

fn default_persist() -> bool {
    true
}