use my_no_sql_core::db::DbTable;
use my_no_sql_server_core::db_snapshots::DbTableSnapshot;

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub table_snapshot: DbTableSnapshot,
}

impl InitTableEventSyncData {
    pub fn new(db_table: &DbTable, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(db_table),
            event_src,
            table_snapshot: DbTableSnapshot::new(db_table.get_last_write_moment(), db_table),
        }
    }
}
