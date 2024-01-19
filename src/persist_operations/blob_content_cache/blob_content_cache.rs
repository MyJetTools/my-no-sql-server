use my_no_sql_sdk::core::db::{
    db_table_master_node::PartitionLastWriteMoment, DbTable, DbTableAttributes,
};
use my_no_sql_server_core::db_snapshots::DbPartitionSnapshot;
use rust_extensions::{date_time::DateTimeAsMicroseconds, sorted_vec::SortedVecWithStrKey};
use tokio::sync::RwLock;

use super::PersistedTableData;

pub enum BlobPartitionUpdateTimeResult {
    Ok(DateTimeAsMicroseconds),
    TableNotFound,
    PartitionNoFound,
}

pub struct BlobContentCache {
    pub data_by_table: RwLock<SortedVecWithStrKey<PersistedTableData>>,
}

impl BlobContentCache {
    pub fn new() -> Self {
        Self {
            data_by_table: RwLock::new(SortedVecWithStrKey::new()),
        }
    }

    pub async fn has_table(&self, table_name: &str) -> bool {
        let read_access = self.data_by_table.read().await;
        read_access.contains(table_name)
    }

    pub async fn init(&self, table_data: &DbTable) {
        let data_to_insert = PersistedTableData::init(table_data);
        let mut write_access = self.data_by_table.write().await;
        write_access.insert_or_replace(data_to_insert);
    }

    pub async fn create_table(&self, table_name: &str, attr: &DbTableAttributes) {
        let data_to_insert = PersistedTableData::new(table_name.to_string(), attr.clone());
        let mut write_access = self.data_by_table.write().await;
        write_access.insert_or_replace(data_to_insert);
    }

    pub async fn update_table_attributes(&self, table_name: &str, attr: DbTableAttributes) {
        let mut write_access = self.data_by_table.write().await;

        match write_access.insert_or_update(table_name) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                let table_data = PersistedTableData::new(table_name.to_string(), attr);
                entry.insert(table_data);
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry.item.attr = attr;
            }
        }
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
        db_partition_snapshot: DbPartitionSnapshot,
    ) {
        let mut write_access = self.data_by_table.write().await;

        let table = write_access.get_mut(table_name);

        if let Some(table) = table {
            table
                .partitions
                .insert_or_replace(PartitionLastWriteMoment {
                    partition_key: db_partition_snapshot.partition_key,
                    last_write_moment: db_partition_snapshot.last_write_moment,
                });
        }
    }

    pub async fn get_snapshot(
        &self,
        table_name: &str,
    ) -> Option<SortedVecWithStrKey<PartitionLastWriteMoment>> {
        let read_access = self.data_by_table.read().await;
        let table = read_access.get(table_name)?;

        let mut result = SortedVecWithStrKey::new();

        for db_partition in table.partitions.iter() {
            result.insert_or_replace(PartitionLastWriteMoment {
                partition_key: db_partition.partition_key.clone(),
                last_write_moment: db_partition.last_write_moment,
            });
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

        BlobPartitionUpdateTimeResult::Ok(result.unwrap().last_write_moment)
    }
}
