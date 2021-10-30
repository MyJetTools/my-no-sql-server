use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use std::collections::{BTreeMap, HashMap};

use crate::db::DbPartition;

use super::db_table_attributes::DbTableAttributes;

pub struct DbTableData {
    pub partitions: BTreeMap<String, DbPartition>,
    pub attributes: DbTableAttributes,
    pub created: DateTimeAsMicroseconds,
    pub last_update_time: AtomicDateTimeAsMicroseconds,
    pub last_read_time: AtomicDateTimeAsMicroseconds,
}

impl DbTableData {
    pub fn new(attributes: DbTableAttributes, now: DateTimeAsMicroseconds) -> Self {
        Self {
            partitions: BTreeMap::new(),
            attributes,
            last_update_time: AtomicDateTimeAsMicroseconds::new(now.unix_microseconds),
            created: now,
            last_read_time: AtomicDateTimeAsMicroseconds::new(now.unix_microseconds),
        }
    }

    pub fn get_or_create_partition(
        &mut self,
        partition_key: &str,
        update_last_access: Option<DateTimeAsMicroseconds>,
    ) -> &mut DbPartition {
        if let Some(partition) = self.partitions.get_mut(partition_key) {
            if let Some(last_access) = update_last_access {
                partition.update_last_access(last_access);
            }

            return partition;
        }

        let result = DbPartition::new();

        self.partitions.insert(partition_key.to_string(), result);

        return self.partitions.get_mut(partition_key).as_mut().unwrap();
    }

    pub fn get_partition(
        &self,
        partition_key: &str,
        update_last_access: Option<DateTimeAsMicroseconds>,
    ) -> Option<&mut DbPartition> {
        let result = DbPartition::new();

        let result = self.partitions.get_mut(partition_key);

        if let Some(result) = result {
            if let Some(last_access) = update_last_access {
                result.update_last_access(last_access);
            }
        }

        result
    }

    pub fn get_partition_mut(
        &mut self,
        partition_key: &str,
        update_last_access: Option<DateTimeAsMicroseconds>,
    ) -> Option<&mut DbPartition> {
        let result = DbPartition::new();

        let result = self.partitions.get_mut(partition_key);

        if let Some(result) = result {
            if let Some(last_access) = update_last_access {
                result.update_last_access(last_access);
            }
        }

        result
    }

    pub fn gc_and_keep_max_partitions_amount(
        &mut self,
        max_partitions_amount: usize,
    ) -> Option<HashMap<String, DbPartition>> {
        if self.partitions.len() <= max_partitions_amount {
            return None;
        }

        let mut partitions_to_gc = BTreeMap::new();

        for (partition_key, partition) in &self.partitions {
            let mut last_read_access = partition.last_read_access.get_unix_microseconds();

            while partitions_to_gc.contains_key(&last_read_access) {
                last_read_access += 1;
            }

            partitions_to_gc.insert(last_read_access, partition_key.to_string());
        }

        let mut result = HashMap::new();

        for (_, partition_key) in partitions_to_gc {
            if self.partitions.len() <= max_partitions_amount {
                break;
            }

            if let Some(partition) = self.partitions.remove(partition_key.as_str()) {
                result.insert(partition_key, partition);
            }
        }

        Some(result)
    }
}
