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
        Self {
            attr: table_data.attributes.clone(),
            partitions: table_data.get_partitions_last_write_moment(),
        }
    }
}
