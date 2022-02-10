use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::db_snapshots::DbRowsSnapshot;

pub struct DbPartitionSnapshot {
    pub last_read_access: DateTimeAsMicroseconds,
    pub last_write_moment: DateTimeAsMicroseconds,
    pub db_rows_snapshot: DbRowsSnapshot,
}
