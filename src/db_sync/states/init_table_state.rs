use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbPartition, DbTable, DbTableSnapshot},
    db_sync::SyncAttributes,
};

pub struct InitTableEventState {
    pub table: Arc<DbTable>,
    pub attrs: SyncAttributes,
    pub cleaned_partitions_before: Option<BTreeMap<String, DbPartition>>,
    pub table_snapshot: Option<DbTableSnapshot>,
}

impl InitTableEventState {
    pub fn new(table: Arc<DbTable>, attrs: SyncAttributes) -> Self {
        Self {
            table,
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
