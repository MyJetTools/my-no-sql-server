use std::collections::BTreeMap;

use crate::{
    db::{DbPartition, DbTable, DbTableSnapshot},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub attrs: SyncAttributes,
    pub cleaned_partitions_before: Option<BTreeMap<String, DbPartition>>,
    pub table_snapshot: Option<DbTableSnapshot>,
}

impl InitTableEventSyncData {
    pub fn new(table: &DbTable, attrs: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(table),
            attrs,
            cleaned_partitions_before: None,
            table_snapshot: None,
        }
    }

    pub fn add_cleaned_partition_before(&mut self, partition_key: String, partition: DbPartition) {
        if self.cleaned_partitions_before.is_none() {
            self.cleaned_partitions_before = Some(BTreeMap::new());
        }

        self.cleaned_partitions_before
            .as_mut()
            .unwrap()
            .insert(partition_key, partition);
    }

    pub fn add_table_snapshot(&mut self, snapshot: DbTableSnapshot) {
        self.table_snapshot = Some(snapshot);
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        return self.table_snapshot.as_ref().unwrap().as_raw_bytes();
    }
}
