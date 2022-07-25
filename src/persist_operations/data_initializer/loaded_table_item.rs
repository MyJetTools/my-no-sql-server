use my_no_sql_core::db::{DbPartition, DbTableAttributesSnapshot};

use crate::{persist_io::TableFile, persist_operations::serializers::TableMetadataFileContract};

pub enum LoadedTableItem {
    TableAttributes(DbTableAttributesSnapshot),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}

impl LoadedTableItem {
    pub fn new(table_file: &TableFile, content: &[u8]) -> Result<Self, String> {
        match table_file {
            TableFile::TableAttributes => {
                let table_metadata = TableMetadataFileContract::parse(content);
                let result = LoadedTableItem::TableAttributes(table_metadata.into());
                return Ok(result);
            }
            TableFile::DbPartition(partition_key) => {
                let db_partition =
                    crate::persist_operations::serializers::db_partition::deserialize(content)?;

                let result = LoadedTableItem::DbPartition {
                    partition_key: partition_key.to_string(),
                    db_partition,
                };

                return Ok(result);
            }
        }
    }
}
