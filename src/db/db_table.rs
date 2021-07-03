use tokio::sync::RwLock;

use crate::db_operations::read_as_json::DbEntityAsJsonArray;

use super::{DbTableAttributes, DbTableData};

pub struct DbTable {
    pub name: String,
    pub created: i64,
    pub data: RwLock<DbTableData>,
}

impl DbTable {
    pub fn new(name: String, data: DbTableData, created: i64) -> Self {
        DbTable {
            created,
            name: name,
            data: RwLock::new(data),
        }
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.get_partitions_amount();
    }

    pub async fn get_partition_rows_amount(&self, partition_key: &str) -> Option<usize> {
        let read_access = self.data.read().await;
        let partition = read_access.get_partition(partition_key)?;

        let count = partition.get_rows_amount();
        return Some(count);
    }

    pub async fn get_attributes(&self) -> DbTableAttributes {
        let read_access = self.data.read().await;

        return read_access.attributes.clone();
    }

    pub async fn get_partition_keys(&self) -> Vec<String> {
        let read_access = self.data.read().await;
        return read_access
            .partitions
            .keys()
            .into_iter()
            .map(|pk| pk.clone())
            .collect();
    }

    pub async fn get_partition_snapshot_as_json(&self, partition_key: &str) -> Option<Vec<u8>> {
        let read_access = self.data.read().await;
        let partition = read_access.get_partition(partition_key)?;

        return Some(partition.as_json_array());
    }
}
