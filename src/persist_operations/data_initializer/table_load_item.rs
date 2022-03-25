use crate::db::{DbPartition, DbTableAttributesSnapshot};

pub enum TableLoadItem {
    TableAttributes(DbTableAttributesSnapshot),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}
