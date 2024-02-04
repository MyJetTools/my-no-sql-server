use my_no_sql_sdk::core::db::DbTableAttributes;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TableMetadataFileContract {
    #[serde(rename = "Persist")]
    #[serde(default = "default_persist")]
    pub persist: bool,
    #[serde(rename = "MaxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
    #[serde(rename = "MaxRowsPerPartitionAmount")]
    pub max_rows_per_partition_amount: Option<usize>,
    #[serde(rename = "Created")]
    pub created: Option<String>,
}

impl TableMetadataFileContract {
    pub fn parse(content: &[u8]) -> Self {
        let parse_result = serde_json::from_slice::<TableMetadataFileContract>(content);

        match parse_result {
            Ok(res) => res,
            Err(_) => TableMetadataFileContract {
                max_partitions_amount: None,
                max_rows_per_partition_amount: None,
                persist: true,
                created: Some(DateTimeAsMicroseconds::now().to_rfc3339()),
            },
        }
    }
}

fn default_persist() -> bool {
    true
}

impl Into<DbTableAttributes> for TableMetadataFileContract {
    fn into(self) -> DbTableAttributes {
        let mut result = DbTableAttributes {
            created: if let Some(created) = &self.created {
                match DateTimeAsMicroseconds::from_str(created) {
                    Some(value) => value,
                    None => DateTimeAsMicroseconds::now(),
                }
            } else {
                DateTimeAsMicroseconds::now()
            },
            max_partitions_amount: self.max_partitions_amount,
            max_rows_per_partition_amount: self.max_rows_per_partition_amount,
            persist: self.persist,
        };

        if let Some(value) = result.max_partitions_amount {
            if value == 0 {
                result.max_partitions_amount = None;
            }
        }

        if let Some(value) = result.max_rows_per_partition_amount {
            if value == 0 {
                result.max_rows_per_partition_amount = None;
            }
        }

        result
    }
}

pub fn serialize(attrs: &DbTableAttributes) -> Vec<u8> {
    let contract = TableMetadataFileContract {
        max_partitions_amount: attrs.max_partitions_amount,
        max_rows_per_partition_amount: attrs.max_rows_per_partition_amount,
        persist: attrs.persist,
        created: Some(attrs.created.to_rfc3339()),
    };

    serde_json::to_vec(&contract).unwrap()
}
