use my_no_sql_sdk::core::db::{DbPartition, DbTableAttributes};

use crate::{persist_io::TableFile, persist_operations::serializers::TableMetadataFileContract};

pub enum LoadedTableItem {
    TableAttributes(DbTableAttributes),
    DbPartition(DbPartition),
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
                    crate::persist_operations::serializers::db_partition::deserialize(
                        partition_key.as_str(),
                        content,
                    )?;
                return Ok(LoadedTableItem::DbPartition(db_partition));
            }
        }
    }
}
