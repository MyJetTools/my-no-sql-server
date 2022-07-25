use std::collections::HashMap;

use my_no_sql_core::db::{DbTableAttributesSnapshot, DbTableInner};
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct PersistedTableData {
    pub attr: DbTableAttributesSnapshot,
    pub partitions: HashMap<String, DateTimeAsMicroseconds>,
}

impl PersistedTableData {
    pub fn new(attr: DbTableAttributesSnapshot) -> Self {
        Self {
            attr,
            partitions: HashMap::new(),
        }
    }

    pub fn init(table_data: &DbTableInner, attr: DbTableAttributesSnapshot) -> Self {
        Self {
            attr,
            partitions: table_data.get_partitions_last_write_moment(),
        }
    }
}
