use crate::db::{DbPartition, DbTableAttributesSnapshot};

pub enum LoadedTableItem {
    TableAttributes(DbTableAttributesSnapshot),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}
