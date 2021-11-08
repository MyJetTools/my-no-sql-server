use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use std::collections::{BTreeMap, HashMap};

use crate::db::DbPartition;

use super::{db_table_attributes::DbTableAttributes, DbTableDataIterator};

pub struct DbTableData {
    pub name: String,
    pub partitions: BTreeMap<String, DbPartition>,
    pub attributes: DbTableAttributes,
    pub created: DateTimeAsMicroseconds,
    pub last_update_time: AtomicDateTimeAsMicroseconds,
    pub last_read_time: AtomicDateTimeAsMicroseconds,
}

impl DbTableData {
    pub fn new(name: String, attributes: DbTableAttributes) -> Self {
        let created = attributes.created;
        Self {
            name,
            partitions: BTreeMap::new(),
            attributes,
            last_update_time: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
            created,
            last_read_time: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
        }
    }

    pub fn get_or_create_partition(&mut self, partition_key: &str) -> &mut DbPartition {
        if !self.partitions.contains_key(partition_key) {
            let result = DbPartition::new();

            self.partitions.insert(partition_key.to_string(), result);
        }

        return self.partitions.get_mut(partition_key).unwrap();
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
            let mut last_read_access = partition.get_last_access().unix_microseconds;

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

    pub fn iterate_all_rows<'s>(&'s self) -> DbTableDataIterator<'s> {
        DbTableDataIterator::new(self)
    }
}
