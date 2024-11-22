use std::sync::Arc;
use std::time::Duration;

use my_no_sql_sdk::core::db::{DbRow, DbTableName, PartitionKey};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use super::persist_markers_inner::PersistMarkersInner;
use super::{PersistMetrics, PersistTask};

pub struct PersistMarkers {
    inner: Mutex<PersistMarkersInner>,
}

impl PersistMarkers {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(PersistMarkersInner::new()),
        }
    }

    pub async fn delete_db_rows(
        &self,
        table_name: &DbTableName,
        partition_key: &PartitionKey,
        persist_moment: DateTimeAsMicroseconds,
        rows_to_delete: impl Iterator<Item = &Arc<DbRow>>,
    ) {
        let mut inner = self.inner.lock().await;
        inner.persist_rows(table_name, partition_key, persist_moment, rows_to_delete);
    }

    pub async fn persist_rows(
        &self,
        table_name: &DbTableName,
        partition_key: &PartitionKey,
        persist_moment: DateTimeAsMicroseconds,
        rows_to_persist: impl Iterator<Item = &Arc<DbRow>>,
    ) {
        let mut inner = self.inner.lock().await;
        inner.persist_rows(table_name, partition_key, persist_moment, rows_to_persist);
    }

    pub async fn persist_partition(
        &self,
        table_name: &DbTableName,
        partition_key: &PartitionKey,
        persist_moment: DateTimeAsMicroseconds,
    ) {
        let mut inner = self.inner.lock().await;
        inner.persist_whole_partition(table_name, partition_key, persist_moment);
    }

    pub async fn persist_table_content(
        &self,
        table_name: &DbTableName,
        persist_moment: DateTimeAsMicroseconds,
    ) {
        let mut inner = self.inner.lock().await;
        inner.persist_table_content(table_name, persist_moment);
    }

    pub async fn persist_table_attributes(
        &self,
        table_name: &DbTableName,
        persist_moment: DateTimeAsMicroseconds,
    ) {
        let mut inner = self.inner.lock().await;
        inner.persist_table_attributes(table_name, persist_moment);
    }

    pub async fn get_persist_task(
        &self,
        now: Option<DateTimeAsMicroseconds>,
    ) -> Option<PersistTask> {
        let mut inner = self.inner.lock().await;

        let result = inner.get_persist_task(now);

        if let Some(result) = result.as_ref() {
            match result {
                PersistTask::SaveTableAttributes(db_table_name) => {
                    inner
                        .get_by_table_mut(db_table_name)
                        .unwrap()
                        .persist_table_attributes = None;
                }
                PersistTask::SyncTable(db_table_name) => {
                    inner
                        .get_by_table_mut(db_table_name)
                        .unwrap()
                        .clean_when_synching_whole_table();
                }

                PersistTask::SyncPartition {
                    table_name,
                    partition_key,
                } => {
                    let by_table = inner.get_by_table_mut(table_name).unwrap();
                    by_table.clean_when_synching_whole_partition(partition_key);
                }
                PersistTask::SyncRows { table_name, jobs } => {
                    let by_table = inner.get_by_table_mut(table_name).unwrap();
                    for job in jobs {
                        by_table.clean_when_syncing_rows(&job.partition_key, &job.items);
                    }
                }
            }
        }

        result
    }

    pub async fn set_last_persist_time(
        &self,
        table_name: &DbTableName,
        now: DateTimeAsMicroseconds,
        duration: Duration,
    ) {
        let mut inner = self.inner.lock().await;
        inner.set_last_persist_time(table_name, now, duration);
    }

    pub async fn get_persist_metrics(&self, table_name: &str) -> PersistMetrics {
        let inner = self.inner.lock().await;
        match inner.get_by_table(table_name) {
            Some(metrics) => metrics.metrics.clone(),
            None => PersistMetrics::default(),
        }
    }

    /*
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
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(
                       entry,
                   ) => {
                       let mut item = PersistByTableItem {
                           table_name: db_table.name.clone(),
                           data: TablePersistData::new(),
                       };

                       item.data
                           .data_to_persist
                           .mark_partition_to_persist(partition_key, sync_moment);

                       entry.insert(item);
                   }
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(
                       entry,
                   ) => {
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
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(
                       entry,
                   ) => {
                       let mut item = PersistByTableItem {
                           table_name: db_table.name.to_string(),
                           data: TablePersistData::new(),
                       };

                       item.data.data_to_persist.mark_table_to_persist(sync_moment);

                       entry.insert(item);
                   }
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(
                       entry,
                   ) => {
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
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(
                       entry,
                   ) => {
                       let mut item = PersistByTableItem {
                           table_name: db_table.name.to_string(),
                           data: TablePersistData::new(),
                       };

                       item.data.data_to_persist.mark_persist_attrs();

                       entry.insert(item);
                   }
                   my_no_sql_sdk::core::rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(
                       entry,
                   ) => {
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
       */
}
