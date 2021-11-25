use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::{
    db::{DbPartitionSnapshot, DbRow},
    json::JsonArrayBuilder,
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

    pub async fn as_json(&self) -> Vec<u8> {
        let mut result = JsonArrayBuilder::new();
        let read_access = self.data.read().await;

        for db_row in read_access.iterate_all_rows() {
            result.append_json_object(&db_row.data);
        }

        result.build()
    }

    pub async fn get_snapshot_as_partitions(&self) -> BTreeMap<String, DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        read_access.get_snapshot_as_partitions()
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        read_access.get_partition_snapshot(partition_key)
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
}
