use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::{DbTableAttributes, DbTableData};

pub struct PersistedTableData {
    pub attr: DbTableAttributes,
    pub partitions: HashMap<String, DateTimeAsMicroseconds>,
}

impl PersistedTableData {
    pub fn new(attr: DbTableAttributes) -> Self {
        Self {
            attr,
            partitions: HashMap::new(),
        }
    }

    pub fn init(table_data: &DbTableData) -> Self {
        let mut partitions = HashMap::new();

        for (partition_key, db_partition) in &table_data.partitions {
            partitions.insert(
                partition_key.to_string(),
                db_partition.last_write_moment.as_date_time(),
            );
        }

        Self {
            attr: table_data.attributes.clone(),
            partitions,
        }
    }
}
