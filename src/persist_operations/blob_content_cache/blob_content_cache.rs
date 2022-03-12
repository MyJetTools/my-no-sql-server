use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::db::{DbTableAttributesSnapshot, DbTableData};

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

    pub async fn init(&self, table_data: &DbTableData, attr: DbTableAttributesSnapshot) {
        let data_to_insert = PersistedTableData::init(table_data, attr);
        let mut write_access = self.data_by_table.write().await;
        write_access.insert(table_data.name.to_string(), data_to_insert);
    }

    pub async fn create_table(&self, table_name: &str, attr: DbTableAttributesSnapshot) {
        let table_data = PersistedTableData::new(attr);
        let mut write_access = self.data_by_table.write().await;
        write_access.insert(table_name.to_string(), table_data);
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
        timestamp: DateTimeAsMicroseconds,
    ) {
        let mut write_access = self.data_by_table.write().await;

        let table = write_access.get_mut(table_name);

        if let Some(table) = table {
            table
                .partitions
                .insert(partition_key.to_string(), timestamp);
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
