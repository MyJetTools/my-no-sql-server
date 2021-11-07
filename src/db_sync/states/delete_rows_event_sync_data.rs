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

    fn check_that_we_are_in_partition_mode(
        &mut self,
        partition_key: &str,
    ) -> &mut BTreeMap<String, BTreeMap<String, Arc<DbRow>>> {
        if let Some(deleted_partitions) = &self.deleted_partitions {
            if deleted_partitions.contains_key(partition_key) {
                panic!("Can not add deleted rows from partition {}", partition_key);
            }
        }

        if self.deleted_rows.is_none() {
            self.deleted_rows = Some(BTreeMap::new())
        }

        return self.deleted_rows.as_mut().unwrap();
    }

    pub fn add_deleted_row(&mut self, partition_key: &str, deleted_row: Arc<DbRow>) {
        let deleted_rows_btree_map = self.check_that_we_are_in_partition_mode(partition_key);

        if !deleted_rows_btree_map.contains_key(partition_key) {
            deleted_rows_btree_map.insert(partition_key.to_string(), BTreeMap::new());
        }

        deleted_rows_btree_map
            .get_mut(partition_key)
            .unwrap()
            .insert(deleted_row.row_key.to_string(), deleted_row.clone());
    }

    pub fn add_deleted_rows(&mut self, partition_key: &str, deleted_rows: &[Arc<DbRow>]) {
        let deleted_rows_btree_map = self.check_that_we_are_in_partition_mode(partition_key);

        if !deleted_rows_btree_map.contains_key(partition_key) {
            deleted_rows_btree_map.insert(partition_key.to_string(), BTreeMap::new());
        }

        let by_partition = deleted_rows_btree_map.get_mut(partition_key).unwrap();

        for deleted_row in deleted_rows {
            by_partition.insert(deleted_row.row_key.to_string(), deleted_row.clone());
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
