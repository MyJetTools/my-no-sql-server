use std::{collections::BTreeMap, sync::Arc};

use crate::{
    db::{DbRow, DbTableData},
    db_sync::EventSource,
};

use super::SyncTableData;

pub struct UpdateRowsSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub updated_rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
}

impl UpdateRowsSyncData {
    pub fn new(table_data: &DbTableData, persist: bool, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, persist),
            event_src,
            updated_rows_by_partition: BTreeMap::new(),
        }
    }

    pub fn add_row(&mut self, db_row: Arc<DbRow>) {
        if !self
            .updated_rows_by_partition
            .contains_key(&db_row.partition_key)
        {
            self.updated_rows_by_partition
                .insert(db_row.partition_key.to_string(), Vec::new());
        }

        self.updated_rows_by_partition
            .get_mut(&db_row.partition_key)
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
