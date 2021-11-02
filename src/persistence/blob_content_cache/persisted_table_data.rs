use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbTableAttributes;

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
}
