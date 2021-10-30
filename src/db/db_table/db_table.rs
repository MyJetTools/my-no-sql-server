use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::{
    db::{DbPartition, DbPartitionSnapshot},
    utils::SortedDictionary,
};

use super::{db_table_attributes::DbTableAttributes, db_table_data::DbTableData};

pub struct DbTable {
    pub name: String,
    pub created: DateTimeAsMicroseconds,
    pub data: RwLock<DbTableData>,
}

impl DbTable {
    pub fn new(name: String, data: DbTableData, created: DateTimeAsMicroseconds) -> Self {
        DbTable {
            created,
            name: name,
            data: RwLock::new(data),
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

        return true;
    }

    pub async fn clear_partition(&self, partition_key: &str) -> bool {
        let mut write_access = self.data.write().await;
        if !write_access.partitions.contains_key(partition_key) {
            return false;
        }

        write_access.partitions.remove(partition_key);
        return true;
    }

    pub async fn gc_partitions(&self, max_partitions_amount: usize) -> Option<Vec<String>> {
        let mut partitions_by_date_time: SortedDictionary<i64, String> = SortedDictionary::new();

        let mut gced = None;

        let mut write_access = self.data.write().await;

        for (partition_key, db_partition) in &write_access.partitions {
            let mut last_access = db_partition.last_read_access.as_date_time();
            let last_access_before_insert = last_access;

            while partitions_by_date_time.contains_key(&last_access.unix_microseconds) {
                last_access.unix_microseconds += 1;
            }

            partitions_by_date_time
                .insert(last_access.unix_microseconds, partition_key.to_string());

            if last_access_before_insert.unix_microseconds != last_access.unix_microseconds {
                db_partition.last_read_access.update(last_access)
            }
        }

        while write_access.partitions.len() > max_partitions_amount {
            let (dt, partition_key) = partitions_by_date_time.first().unwrap();

            let removed_result = write_access.partitions.remove(&partition_key);

            if let Some(_) = removed_result {
                if gced.is_none() {
                    gced = Some(Vec::new())
                }

                gced.as_mut().unwrap().push(partition_key);
            }

            partitions_by_date_time.remove(&dt);
        }

        gced
    }

    pub async fn clear(&self) -> bool {
        let mut write_access = self.data.write().await;
        if write_access.partitions.len() == 0 {
            return false;
        }

        write_access.partitions.clear();
        return true;
    }

    pub async fn remove_partition(&self, partition_key: &str) -> Option<DbPartition> {
        let mut write_access = self.data.write().await;
        write_access.partitions.remove(partition_key)
    }

    pub async fn init_partition(&self, partition_key: String, partition: DbPartition) {
        let mut write_access = self.data.write().await;
        write_access.partitions.insert(partition_key, partition);
    }

    pub async fn get_snapshot_as_partitions(
        &self,
        update_read_access_time: Option<DateTimeAsMicroseconds>,
    ) -> Vec<DbPartitionSnapshot> {
        let mut result = Vec::new();
        let read_access = self.data.read().await;

        for db_partition in read_access.partitions.values() {
            let partition_snapshot =
                db_partition.get_db_partition_snapshot(update_read_access_time);
            result.push(partition_snapshot);
        }

        result
    }
}
