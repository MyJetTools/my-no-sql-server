use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::{date_time::MyDateTime, db_operations::read_as_json::DbEntityAsJsonArray};

use super::{
    db_table_snapshot::DbTableSnapshot, DbPartitionSnapshot, DbTableAttributes, DbTableData,
};

pub struct DbTable {
    pub name: String,
    pub created: MyDateTime,
    pub data: RwLock<DbTableData>,
}

impl DbTable {
    pub fn new(name: String, data: DbTableData, created: MyDateTime) -> Self {
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

    pub async fn get_partition_update_time(&self, partition_key: &str) -> Option<i64> {
        let read_access = self.data.read().await;
        let partition = read_access.get_partition(partition_key)?;

        return Some(partition.last_updated.get());
    }

    pub async fn get_snapshot(&self) -> DbTableSnapshot {
        let read_access = self.data.read().await;

        let mut partitions = HashMap::new();

        for (partition_key, partition) in &read_access.partitions {
            let content = partition.as_json_array();

            let partition_snapshot = DbPartitionSnapshot {
                last_update: partition.last_updated.get(),
                content,
            };

            partitions.insert(partition_key.to_string(), partition_snapshot);
        }

        DbTableSnapshot {
            attr: read_access.attributes.clone(),
            created: self.created.miliseconds,
            partitions,
        }
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;

        let db_partition = read_access.partitions.get(partition_key)?;

        let result = DbPartitionSnapshot {
            last_update: db_partition.last_updated.get(),
            content: db_partition.as_json_array(),
        };

        Some(result)
    }
}
