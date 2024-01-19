use std::time::Duration;

use my_no_sql_sdk::core::db::{DbTable, PartitionKeyParameter};
use rust_extensions::{
    date_time::DateTimeAsMicroseconds,
    sorted_vec::{EntityWithStrKey, SortedVecWithStrKey},
};
use tokio::sync::Mutex;

use super::{partition_persist_marker::PersistResult, TablePersistData};

pub struct PersistMetrics {
    pub last_persist_time: Option<DateTimeAsMicroseconds>,
    pub next_persist_time: Option<DateTimeAsMicroseconds>,
    pub persist_amount: usize,
    pub last_persist_duration: Vec<usize>,
}

pub struct PersistByTableItem {
    pub table_name: String,
    pub data: TablePersistData,
}

impl EntityWithStrKey for PersistByTableItem {
    fn get_key(&self) -> &str {
        self.table_name.as_str()
    }
}

pub struct PersistMarkersByTable {
    by_table: Mutex<SortedVecWithStrKey<PersistByTableItem>>,
}

impl PersistMarkersByTable {
    pub fn new() -> Self {
        Self {
            by_table: Mutex::new(SortedVecWithStrKey::new()),
        }
    }

    pub async fn persist_partition(
        &self,
        db_table: &DbTable,
        partition_key: &impl PartitionKeyParameter,
        sync_moment: DateTimeAsMicroseconds,
    ) {
        if !db_table.attributes.persist {
            return;
        }

        let mut write_access = self.by_table.lock().await;

        match write_access.insert_or_update(db_table.name.as_str()) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                let mut item = PersistByTableItem {
                    table_name: db_table.name.clone(),
                    data: TablePersistData::new(),
                };

                item.data
                    .data_to_persist
                    .mark_partition_to_persist(partition_key, sync_moment);

                entry.insert(item);
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry
                    .item
                    .data
                    .data_to_persist
                    .mark_partition_to_persist(partition_key, sync_moment);
            }
        }
    }

    pub async fn persist_table(&self, db_table: &DbTable, sync_moment: DateTimeAsMicroseconds) {
        if !db_table.attributes.persist {
            return;
        }

        let mut write_access = self.by_table.lock().await;

        match write_access.insert_or_update(&db_table.name) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                let mut item = PersistByTableItem {
                    table_name: db_table.name.to_string(),
                    data: TablePersistData::new(),
                };

                item.data.data_to_persist.mark_table_to_persist(sync_moment);

                entry.insert(item);
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry
                    .item
                    .data
                    .data_to_persist
                    .mark_table_to_persist(sync_moment);
            }
        }
    }

    pub async fn persist_table_attrs(&self, db_table: &DbTable) {
        if !db_table.attributes.persist {
            return;
        }

        let mut write_access = self.by_table.lock().await;

        match write_access.insert_or_update(&db_table.name) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                let mut item = PersistByTableItem {
                    table_name: db_table.name.to_string(),
                    data: TablePersistData::new(),
                };

                item.data.data_to_persist.mark_persist_attrs();

                entry.insert(item);
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry.item.data.data_to_persist.mark_persist_attrs();
            }
        }
    }

    pub async fn get_job_to_persist(
        &self,
        table_name: &str,
        now: DateTimeAsMicroseconds,
        is_shutting_down: bool,
    ) -> Option<PersistResult> {
        let mut write_access = self.by_table.lock().await;

        let item = write_access.get_mut(table_name)?;

        item.data
            .data_to_persist
            .get_what_to_persist(now, is_shutting_down)
    }

    pub async fn set_persisted(&self, table_name: &str, duration: Duration) {
        let mut write_access = self.by_table.lock().await;

        if let Some(item) = write_access.get_mut(table_name) {
            item.data.add_persist_duration(duration);
        }
    }

    pub async fn get_persist_metrics(&self, table_name: &str) -> PersistMetrics {
        let read_access = self.by_table.lock().await;

        match read_access.get(table_name) {
            Some(result) => PersistMetrics {
                last_persist_time: result.data.last_persist_time.clone(),
                next_persist_time: result.data.data_to_persist.get_next_persist_time(),
                persist_amount: result.data.data_to_persist.get_persist_amount(),
                last_persist_duration: result.data.persist_duration.clone(),
            },
            None => PersistMetrics {
                last_persist_time: None,
                next_persist_time: None,
                persist_amount: 0,
                last_persist_duration: vec![],
            },
        }
    }
}
