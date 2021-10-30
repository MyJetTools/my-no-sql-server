use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbPartitionSnapshot, DbTable},
    db_sync::SyncAttributes,
};

pub struct UpdatePartitionsState {
    pub table: Arc<DbTable>,
    pub attr: SyncAttributes,
    pub partitions_to_update: BTreeMap<String, Option<DbPartitionSnapshot>>,
}

impl UpdatePartitionsState {
    pub fn new(table: Arc<DbTable>, attr: SyncAttributes) -> Self {
        Self {
            table,
            attr,
            partitions_to_update: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, partition_key: String, snapshot: Option<DbPartitionSnapshot>) {
        self.partitions_to_update.insert(partition_key, snapshot);
    }
}
