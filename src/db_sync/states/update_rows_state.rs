use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbRow, DbTable},
    db_sync::SyncAttributes,
};

pub struct UpdateRowsSyncState {
    pub table: Arc<DbTable>,
    pub attr: SyncAttributes,
    pub updated_rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
}

impl UpdateRowsSyncState {
    pub fn new(table: Arc<DbTable>, attr: SyncAttributes) -> Self {
        Self {
            table,
            attr,
            updated_rows_by_partition: BTreeMap::new(),
        }
    }

    pub fn add_row(&mut self, partition_key: &str, db_row: Arc<DbRow>) {
        if !self.updated_rows_by_partition.contains_key(partition_key) {
            self.updated_rows_by_partition
                .insert(partition_key.to_string(), Vec::new());
        }

        self.updated_rows_by_partition
            .get_mut(partition_key)
            .as_mut()
            .unwrap()
            .push(db_row);
    }

    pub fn add_rows(&mut self, partition_key: &str, db_rows: Vec<Arc<DbRow>>) {
        if !self.updated_rows_by_partition.contains_key(partition_key) {
            self.updated_rows_by_partition
                .insert(partition_key.to_string(), db_rows);
        } else {
            self.updated_rows_by_partition
                .get_mut(partition_key)
                .as_mut()
                .unwrap()
                .extend(db_rows);
        }
    }
}
