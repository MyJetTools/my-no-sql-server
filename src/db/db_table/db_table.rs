use std::{collections::HashMap, sync::atomic::AtomicBool};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::db::DbPartitionSnapshot;

use super::{db_table_attributes::DbTableAttributes, db_table_data::DbTableData};

pub struct DbTable {
    pub name: String,
    pub created: DateTimeAsMicroseconds,
    pub data: RwLock<DbTableData>,
    persist: AtomicBool,
}

impl DbTable {
    pub fn new(name: String, data: DbTableData, created: DateTimeAsMicroseconds) -> Self {
        let persist = AtomicBool::new(data.attributes.persist);
        DbTable {
            created,
            name: name,
            data: RwLock::new(data),
            persist,
        }
    }

    pub async fn get_partitions_amount(&self) -> usize {
        let read_access = self.data.read().await;
        return read_access.partitions.len();
    }

    pub async fn get_attributes(&self) -> DbTableAttributes {
        let read_access = self.data.read().await;

        return read_access.attributes.clone();
    }

    pub async fn set_table_attributes(
        &self,
        persist_table: bool,
        max_partitions_amount: Option<usize>,
    ) -> bool {
        let mut write_access = self.data.write().await;
        if write_access.attributes.persist == persist_table
            && write_access.attributes.max_partitions_amount == max_partitions_amount
        {
            return false;
        }

        write_access.attributes.persist = persist_table;
        write_access.attributes.max_partitions_amount = max_partitions_amount;
        self.persist
            .store(persist_table, std::sync::atomic::Ordering::SeqCst);

        return true;
    }

    pub fn get_persist(&self) -> bool {
        return self.persist.load(std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn get_snapshot_as_partitions(&self) -> HashMap<String, DbPartitionSnapshot> {
        let mut result = HashMap::new();
        let read_access = self.data.read().await;

        for (partition_key, db_partition) in &read_access.partitions {
            let partition_snapshot = db_partition.get_db_partition_snapshot();
            result.insert(partition_key.to_string(), partition_snapshot);
        }

        result
    }

    pub async fn get_partition_snapshot(&self, partition_key: &str) -> Option<DbPartitionSnapshot> {
        let read_access = self.data.read().await;

        let partition = read_access.partitions.get(partition_key)?;

        return Some(partition.get_db_partition_snapshot());
    }
}
