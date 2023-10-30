use std::collections::HashMap;

use my_no_sql_sdk::core::db::{DbTable, DbTableAttributes};
use my_no_sql_server_core::db_snapshots::DbPartitionSnapshot;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use super::PersistedTableData;

pub enum BlobPartitionUpdateTimeResult {
    Ok(DateTimeAsMicroseconds),
    TableNotFound,
    PartitionNoFound,
}

pub struct BlobContentCache {
    pub data_by_table: RwLock<HashMap<String, PersistedTableData>>,
}

impl BlobContentCache {
    pub fn new() -> Self {
        Self {
            data_by_table: RwLock::new(HashMap::new()),
        }
    }

    pub async fn has_table(&self, table_name: &str) -> bool {
        let read_access = self.data_by_table.read().await;
        read_access.contains_key(table_name)
    }

    pub async fn init(&self, table_data: &DbTable) {
        let data_to_insert = PersistedTableData::init(table_data);
        let mut write_access = self.data_by_table.write().await;
        write_access.insert(table_data.name.to_string(), data_to_insert);
    }

    pub async fn create_table(&self, table_name: &str, attr: &DbTableAttributes) {
        let table_data = PersistedTableData::new(attr.clone());
        let mut write_access = self.data_by_table.write().await;
        write_access.insert(table_name.to_string(), table_data);
    }

    pub async fn update_table_attributes(&self, table_name: &str, attr: DbTableAttributes) {
        let mut write_access = self.data_by_table.write().await;

        if !write_access.contains_key(table_name) {
            let table_data = PersistedTableData::new(attr.clone());
            write_access.insert(table_name.to_string(), table_data);
        }

        let data = write_access.get_mut(table_name).unwrap();
        data.attr = attr;
    }

    pub async fn delete_table(&self, table_name: &str) {
        let mut write_access = self.data_by_table.write().await;
        write_access.remove(table_name);
    }

    pub async fn delete_table_partition(&self, table_name: &str, partition_key: &str) {
        let mut write_access = self.data_by_table.write().await;

        let table = write_access.get_mut(table_name);

        if let Some(table) = table {
            table.partitions.remove(partition_key);
        }
    }

    pub async fn update_table_partition_snapshot_id(
        &self,
        table_name: &str,
        partition_key: &str,
        db_partition_snapshot: &DbPartitionSnapshot,
    ) {
        let mut write_access = self.data_by_table.write().await;

        let table = write_access.get_mut(table_name);

        if let Some(table) = table {
            table.partitions.insert(
                partition_key.to_string(),
                db_partition_snapshot.last_write_moment,
            );
        }
    }

    pub async fn get_snapshot(
        &self,
        table_name: &str,
    ) -> Option<HashMap<String, DateTimeAsMicroseconds>> {
        let read_access = self.data_by_table.read().await;
        let table = read_access.get(table_name)?;

        let mut result = HashMap::new();

        for (partition, value) in &table.partitions {
            result.insert(partition.to_string(), *value);
        }

        Some(result)
    }

    pub async fn get(
        &self,
        table_name: &str,
        partition_key: &str,
    ) -> BlobPartitionUpdateTimeResult {
        let read_access = self.data_by_table.read().await;

        let table = read_access.get(table_name);

        if table.is_none() {
            return BlobPartitionUpdateTimeResult::TableNotFound;
        }

        let table = table.unwrap();

        let result = table.partitions.get(partition_key);

        if result.is_none() {
            return BlobPartitionUpdateTimeResult::PartitionNoFound;
        }

        BlobPartitionUpdateTimeResult::Ok(*result.unwrap())
    }
}
