use my_no_sql_sdk::core::db::DbTable;

use crate::db_sync::EventSource;

use super::SyncTableData;

pub struct DeleteTableSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
}

impl DeleteTableSyncData {
    pub fn new(db_table: &DbTable, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(db_table),
            event_src,
        }
    }
}
