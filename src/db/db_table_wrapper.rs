use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use my_json::json_writer::JsonArrayWriter;
use my_no_sql_core::db::{
    db_snapshots::{DbPartitionSnapshot, DbTableSnapshot},
    DbRow, DbTable, DbTableAttributes,
};
use tokio::sync::RwLock;

use crate::persist::PersistMarkers;

pub struct DbTableSingleThreaded {
    pub db_table: DbTable,
    pub persist_markers: PersistMarkers,
}

pub struct DbTableWrapper {
    pub name: String,
    pub data: RwLock<DbTableSingleThreaded>,
}

impl DbTableWrapper {
    pub fn new(db_table: DbTable) -> Arc<Self> {
        let result = Self {
            name: db_table.name.clone(),
            data: RwLock::new(DbTableSingleThreaded {
                db_table,
                persist_markers: PersistMarkers::new(),
            }),
        };

        Arc::new(result)
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;
        let db_partition = read_access.db_table.get_partition(partition_key)?;
        let result: DbPartitionSnapshot = db_partition.into();
        result.into()
    }

    pub async fn get_table_as_json_array(&self) -> JsonArrayWriter {
        let read_access = self.data.read().await;
        read_access.db_table.get_table_as_json_array()
    }

    pub async fn get_all_as_vec_dequeue(&self) -> VecDeque<Arc<DbRow>> {
        let read_access = self.data.read().await;

        let mut result = VecDeque::new();

        for db_row in read_access.db_table.get_all_rows() {
            result.push_back(db_row.clone());
        }

        result
    }

    pub async fn get_table_snapshot(&self) -> DbTableSnapshot {
        let read_access = self.data.read().await;

        DbTableSnapshot {
            attr: read_access.db_table.attributes.clone(),
            last_update_time: read_access.db_table.get_last_update_time(),
            by_partition: get_partitions_snapshot(&read_access.db_table),
        }
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        read_access.db_table.partitions.len()
    }

    pub async fn get_table_attributes(&self) -> DbTableAttributes {
        let read_access = self.data.read().await;
        read_access.db_table.attributes.clone()
    }

    pub async fn get_persist_attr(&self) -> bool {
        let read_access = self.data.read().await;
        read_access.db_table.attributes.persist
    }

    pub async fn get_table_size(&self) -> usize {
        let read_access = self.data.read().await;
        read_access.db_table.get_table_size()
    }
}

fn get_partitions_snapshot(db_table: &DbTable) -> BTreeMap<String, DbPartitionSnapshot> {
    let mut result = BTreeMap::new();

    for (partition_key, db_partition) in &db_table.partitions {
        result.insert(partition_key.to_string(), db_partition.into());
    }

    result
}
