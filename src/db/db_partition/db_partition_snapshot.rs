use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbRow;

pub struct DbPartitionSnapshot {
    pub last_read_access: DateTimeAsMicroseconds,
    pub last_write_access: DateTimeAsMicroseconds,
    pub content: Vec<Arc<DbRow>>,
}
