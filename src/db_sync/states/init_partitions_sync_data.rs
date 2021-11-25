use std::collections::BTreeMap;

use crate::{
    db::{DbPartitionSnapshot, DbTableData},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct InitPartitionsSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl InitPartitionsSyncData {
    pub fn new(table_data: &DbTableData, attr: SyncAttributes, persist: bool) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, persist),
            attr,
            partitions_to_update: BTreeMap::new(),
        }
    }

    pub fn new_as_update_partition(
        table_data: &DbTableData,
        partition_key: &str,
        attr: SyncAttributes,
        persist: bool,
    ) -> Self {
        let mut partitions_to_update = BTreeMap::new();

        let partition_snapshot = table_data.get_partition_snapshot(partition_key);

        partitions_to_update.insert(partition_key.to_string(), partition_snapshot);

        Self {
            table_data: SyncTableData::new(table_data, persist),
            attr,
            partitions_to_update,
        }
    }

    pub fn add(&mut self, partition_key: String, snapshot: Option<DbPartitionSnapshot>) {
        self.partitions_to_update.insert(partition_key, snapshot);
    }
}
