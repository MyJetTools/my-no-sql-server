use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::{DbTableAttributesSnapshot, DbTableData};

pub struct PersistedTableData {
    pub attr: DbTableAttributesSnapshot,
    pub partitions: HashMap<String, DateTimeAsMicroseconds>,
}

impl PersistedTableData {
    pub fn new(attr: &DbTableAttributesSnapshot) -> Self {
        Self {
            attr: attr.clone(),
            partitions: HashMap::new(),
        }
    }

    pub fn init(table_data: &DbTableData, attr: DbTableAttributesSnapshot) -> Self {
        Self {
            attr,
            partitions: table_data.get_partitions_last_write_moment(),
        }
    }
}
