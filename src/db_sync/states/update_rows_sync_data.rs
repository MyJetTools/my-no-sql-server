use my_no_sql_sdk::core::db::DbTable;
use my_no_sql_server_core::db_snapshots::DbRowsByPartitionsSnapshot;

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct UpdateRowsSyncData {
    pub table_data: SyncTableData,
    #[allow(dead_code)]
    pub event_src: EventSource,
    pub rows_by_partition: DbRowsByPartitionsSnapshot,
}

impl UpdateRowsSyncData {
    pub fn new(db_table: &DbTable, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(db_table),
            event_src,
            rows_by_partition: DbRowsByPartitionsSnapshot::new(),
        }
    }
}
