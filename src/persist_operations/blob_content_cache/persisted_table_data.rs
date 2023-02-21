use std::collections::HashMap;

use my_no_sql_core::db::{DbTable, DbTableAttributes};
use rust_extensions::date_time::DateTimeAsMicroseconds;

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

    pub fn init(db_table: &DbTable) -> Self {
        Self {
            attr: db_table.attributes.clone(),
            partitions: db_table.get_partitions_last_write_moment(),
        }
    }
}
