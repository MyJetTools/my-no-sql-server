use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbPartition, DbRow, DbTable},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct DeleteRowsEventSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
    pub deleted_partitions: Option<BTreeMap<String, DbPartition>>,
    pub deleted_rows: Option<BTreeMap<String, BTreeMap<String, Arc<DbRow>>>>,
}

impl DeleteRowsEventSyncData {
    pub fn new(table: &DbTable, attr: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(table),
            attr,
            deleted_partitions: None,
            deleted_rows: None,
        }
    }

    pub fn add_deleted_rows(&mut self, partition_key: &str, deleted_rows: &[Arc<DbRow>]) {
        if let Some(deleted_partitions) = &self.deleted_partitions {
            if deleted_partitions.contains_key(partition_key) {
                panic!(
                    "Can not add deleted rows from partition {}. Amount{}",
                    partition_key,
                    deleted_rows.len()
                );
            }
        }

        if self.deleted_rows.is_none() {
            self.deleted_rows = Some(BTreeMap::new())
        }

        let deleted_rows_btree_map = self.deleted_rows.as_mut().unwrap();

        for db_row in deleted_rows {
            if !deleted_rows_btree_map.contains_key(partition_key) {
                deleted_rows_btree_map.insert(db_row.row_key.to_string(), BTreeMap::new());
            }

            deleted_rows_btree_map
                .get_mut(partition_key)
                .as_mut()
                .unwrap()
                .insert(db_row.row_key.to_string(), db_row.clone());
        }
    }

    pub fn new_deleted_partition(&mut self, partition_key: String, partition: DbPartition) {
        if let Some(deleted_rows) = &mut self.deleted_rows {
            deleted_rows.remove(partition_key.as_str());
        }

        if self.deleted_partitions.is_none() {
            self.deleted_partitions = Some(BTreeMap::new());
        }

        self.deleted_partitions
            .as_mut()
            .unwrap()
            .insert(partition_key, partition);
    }
}
