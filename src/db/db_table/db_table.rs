use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use my_json::json_writer::JsonArrayWriter;
use rust_extensions::{
    date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds},
    MyTimer,
};
use tokio::sync::{Mutex, RwLock};

use crate::{
    app::RequestMetrics,
    db::{
        db_snapshots::{DbPartitionSnapshot, DbTableSnapshot},
        DbRow,
    },
    persist_operations::data_to_persist::PersistResult,
};

use super::{
    db_table_attributes::DbTableAttributes, db_table_data::DbTableData, DbTableAttributesSnapshot,
};

pub struct DbTable {
    pub name: String,
    pub data: RwLock<DbTableData>,
    pub attributes: DbTableAttributes,
    last_update_time: AtomicDateTimeAsMicroseconds,

    pub common_persist_thread: AtomicBool,
    pub dedicated_thread: Mutex<Option<MyTimer>>,

    pub request_metrics: RequestMetrics,
}

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
    pub persist_amount: usize,
    pub records_amount: usize,
    pub expiration_index_records_amount: usize,
    pub last_update_time: DateTimeAsMicroseconds,
    pub last_persist_time: DateTimeAsMicroseconds,
    pub next_persist_time: Option<DateTimeAsMicroseconds>,
    pub last_persist_duration: Vec<usize>,
}

impl DbTable {
    pub fn new(data: DbTableData, attributes: DbTableAttributesSnapshot) -> Self {
        let created = data.created.unix_microseconds;
        DbTable {
            attributes: attributes.into(),
            name: data.name.to_string(),
            data: RwLock::new(data),
            last_update_time: AtomicDateTimeAsMicroseconds::new(created),
            common_persist_thread: AtomicBool::new(true),
            dedicated_thread: Mutex::new(None),
            request_metrics: RequestMetrics::new(),
        }
    }

    pub async fn get_metrics(&self) -> DbTableMetrics {
        let read_access = self.data.read().await;
        read_access.get_metrics(self)
    }

    pub fn get_last_update_time(&self) -> DateTimeAsMicroseconds {
        self.last_update_time.as_date_time()
    }

    pub fn set_last_update_time(&self, value: DateTimeAsMicroseconds) {
        self.last_update_time.update(value);
    }

    pub async fn get_table_size(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.get_calculated_metrics().data_size;
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.get_partitions_amount();
    }

    pub async fn get_table_as_json_array(&self) -> JsonArrayWriter {
        let read_access = self.data.read().await;
        read_access.get_table_as_json_array()
    }

    pub async fn get_all_as_vec_dequeue(&self) -> VecDeque<Arc<DbRow>> {
        let read_access = self.data.read().await;

        let mut result = VecDeque::new();

        for db_row in read_access.get_all_rows() {
            result.push_back(db_row.clone());
        }

        result
    }

    pub async fn update_last_persist_time(&self, success: bool, duration: Duration) {
        let mut write_access = self.data.write().await;
        if success {
            write_access.update_last_persist_time();
        }

        write_access.update_last_persist_duration(duration)
    }

    pub fn persist_using_common_thread(&self) -> bool {
        self.common_persist_thread.load(Ordering::Relaxed)
    }

    pub async fn get_what_to_persist(&self, is_shutting_down: bool) -> Option<PersistResult> {
        let now = DateTimeAsMicroseconds::now();
        let mut write_access = self.data.write().await;
        write_access
            .data_to_persist
            .get_what_to_persist(now, is_shutting_down)
    }

    pub async fn get_table_snapshot(&self) -> DbTableSnapshot {
        let last_update_time = self.get_last_update_time();
        let read_access = self.data.read().await;
        let read_access: &DbTableData = &read_access;

        DbTableSnapshot {
            attr: self.attributes.get_snapshot(),
            created: read_access.created,
            last_update_time,
            by_partition: read_access.into(),
        }
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        let db_partition = read_access.get_partition(partition_key)?;
        let result: DbPartitionSnapshot = db_partition.into();
        result.into()
    }
}
