use std::collections::BTreeMap;

use my_no_sql_core::db::{DbTable, DbTableAttributes};
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub struct PersistedTableData {
    pub attr: DbTableAttributes,
    pub partitions: BTreeMap<String, DateTimeAsMicroseconds>,
}

impl PersistedTableData {
    pub fn new(attr: DbTableAttributes) -> Self {
        Self {
            attr,
            partitions: BTreeMap::new(),
        }
    }

    pub fn init(db_table: &DbTable) -> Self {
        Self {
            attr: db_table.attributes.clone(),
            partitions: db_table.get_partitions_last_write_moment(),
        }
    }
}
