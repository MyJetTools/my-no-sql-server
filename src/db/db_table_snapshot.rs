use std::collections::HashMap;

use super::{DbPartitionSnapshot, DbTableAttributes};

pub struct DbTableSnapshot {
    pub attr: DbTableAttributes,
    pub created: i64,
    pub partitions: HashMap<String, DbPartitionSnapshot>,
}
