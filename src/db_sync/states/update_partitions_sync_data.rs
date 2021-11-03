use std::collections::BTreeMap;

use crate::{
    db::{DbPartitionSnapshot, DbTable},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct UpdatePartitionsSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl UpdatePartitionsSyncData {
    pub fn new(table: &DbTable, attr: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(table),
            attr,
            partitions_to_update: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, partition_key: String, snapshot: Option<DbPartitionSnapshot>) {
        self.partitions_to_update.insert(partition_key, snapshot);
    }
}
