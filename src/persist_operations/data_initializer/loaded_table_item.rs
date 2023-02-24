use my_no_sql_core::db::{DbPartition, DbTableAttributes};

use crate::{persist_io::TableFile, persist_operations::serializers::TableMetadataFileContract};

pub enum LoadedTableItem {
    TableAttributes(DbTableAttributes),
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
                    crate::persist_operations::serializers::db_partition::deserialize_from_io(
                        content,
                    )?;

                let result = LoadedTableItem::DbPartition {
                    partition_key: partition_key.to_string(),
                    db_partition,
                };

                return Ok(result);
            }
        }
    }
}
