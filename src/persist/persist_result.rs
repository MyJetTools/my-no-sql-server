use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum PersistResult {
    PersistAttrs,
    PersistTable(DateTimeAsMicroseconds),
    PersistPartition {
        partition_key: String,
        persist_moment: DateTimeAsMicroseconds,
    },
    PersistRows {
        partition_key: String,
        row_keys: HashMap<String, DateTimeAsMicroseconds>,
    },
}

impl PersistResult {
    pub fn get_partition_key(&self) -> Option<&str> {
        match self {
            PersistResult::PersistAttrs => None,
            PersistResult::PersistTable(_) => None,
            PersistResult::PersistPartition {
                partition_key,
                persist_moment: _,
            } => Some(partition_key.as_str()),
            PersistResult::PersistRows {
                partition_key,
                row_keys: _,
            } => Some(partition_key.as_str()),
        }
    }
}
