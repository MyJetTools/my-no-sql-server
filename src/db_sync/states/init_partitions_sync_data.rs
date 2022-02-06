use std::collections::BTreeMap;

use crate::{
    db::{db_snapshots::DbPartitionSnapshot, DbTableData},
    db_sync::EventSource,
};

use super::SyncTableData;

pub struct InitPartitionsSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl InitPartitionsSyncData {
    pub fn new(table_data: &DbTableData, event_src: EventSource, persist: bool) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, persist),
            event_src,
            partitions_to_update: BTreeMap::new(),
        }
    }

    pub fn new_as_update_partition(
        table_data: &DbTableData,
        partition_key: &str,
        event_src: EventSource,
        persist: bool,
    ) -> Self {
        let mut partitions_to_update = BTreeMap::new();

        if let Some(db_partition) = table_data.get_partition(partition_key) {
            let partition_snapshot: DbPartitionSnapshot = db_partition.into();
            partitions_to_update.insert(partition_key.to_string(), Some(partition_snapshot));
        } else {
            partitions_to_update.insert(partition_key.to_string(), None);
        }

        Self {
            table_data: SyncTableData::new(table_data, persist),
            event_src,
            partitions_to_update,
        }
    }

    pub fn add(&mut self, partition_key: String, snapshot: Option<DbPartitionSnapshot>) {
        self.partitions_to_update.insert(partition_key, snapshot);
    }
}
