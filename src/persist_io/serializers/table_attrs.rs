use crate::db::DbTableAttributesSnapshot;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

pub fn serialize(attr: &DbTableAttributesSnapshot) -> Vec<u8> {
    let contract = TableMetadataFileContract {
        persist: attr.persist,
        max_partitions_amount: attr.max_partitions_amount,
    };

    serde_json::to_vec(&contract).unwrap()
}

pub const METADATA_FILE_NAME: &str = ".metadata";

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

impl Into<DbTableAttributesSnapshot> for TableMetadataFileContract {
    fn into(self) -> DbTableAttributesSnapshot {
        DbTableAttributesSnapshot {
            created: DateTimeAsMicroseconds::now(),
            max_partitions_amount: self.max_partitions_amount,
            persist: self.persist,
        }
    }
}
