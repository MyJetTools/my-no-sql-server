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
                        .get_by_table_mut(&db_table_name)
                        .unwrap()
                        .persist_table_attributes = None;
                }
                PersistTask::SyncTable(db_table_name) => {
                    inner
                        .get_by_table_mut(&db_table_name)
                        .unwrap()
                        .clean_when_synching_whole_table();
                }

                PersistTask::SyncPartition {
                    table_name,
                    partition_key,
                } => {
                    let by_table = inner.get_by_table_mut(&table_name).unwrap();
                    by_table.clean_when_synching_whole_partition(&partition_key);
                }
                PersistTask::SyncRows { table_name, jobs } => {
                    let by_table = inner.get_by_table_mut(&table_name).unwrap();
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
            Some(metrics) => metrics.get_metrics(),
            None => PersistMetrics::default(),
        }
    }

    pub async fn has_something_to_persist(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.has_something_to_persist()
    }
}
