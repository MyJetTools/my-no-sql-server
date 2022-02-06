use std::{collections::VecDeque, sync::Arc};

use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::{
    db::{
        db_snapshots::{DbPartitionSnapshot, DbTableSnapshot},
        DbRow,
    },
    persistence::PersistResult,
};

use super::{
    db_table_attributes::DbTableAttributes, db_table_data::DbTableData, DbTableAttributesSnapshot,
};

pub struct DbTable {
    pub name: String,
    pub data: RwLock<DbTableData>,
    pub attributes: DbTableAttributes,
}

pub struct DbTableMetrics {
    pub table_size: usize,
    pub partitions_amount: usize,
}

impl DbTable {
    pub fn new(data: DbTableData, attributes: DbTableAttributesSnapshot) -> Self {
        DbTable {
            attributes: attributes.into(),
            name: data.name.to_string(),
            data: RwLock::new(data),
        }
    }

    pub async fn get_metrics(&self) -> DbTableMetrics {
        let read_access = self.data.read().await;

        return DbTableMetrics {
            table_size: read_access.table_size,
            partitions_amount: read_access.get_partitions_amount(),
        };
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.get_partitions_amount();
    }

    pub async fn get_table_as_json_array(&self) -> JsonArrayWriter {
        let read_access = self.data.read().await;
        read_access.get_table_as_json_array()
    }

    pub async fn get_expired_rows(&self, now: DateTimeAsMicroseconds) -> Option<Vec<Arc<DbRow>>> {
        let mut write_access = self.data.write().await;
        write_access.get_expired_rows_up_to(now)
    }

    pub async fn get_all_as_vec_dequeue(&self) -> VecDeque<Arc<DbRow>> {
        let read_access = self.data.read().await;

        let mut result = VecDeque::new();

        for db_row in read_access.iterate_all_rows() {
            result.push_back(db_row.clone());
        }

        result
    }

    pub async fn get_what_to_persist(&self, is_shutting_down: bool) -> Option<PersistResult> {
        let now = DateTimeAsMicroseconds::now();
        let mut write_access = self.data.write().await;
        write_access
            .data_to_persist
            .get_what_to_persist(now, is_shutting_down)
    }

    pub async fn get_table_snapshot(&self) -> DbTableSnapshot {
        let read_access = self.data.read().await;
        let read_access: &DbTableData = &read_access;

        DbTableSnapshot {
            attr: self.attributes.get_snapshot(),
            created: read_access.created,
            last_update: read_access.last_update_time.as_date_time(),
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
