use crate::{db::DbTableData, db_sync::EventSource};

use super::SyncTableData;

pub struct DeleteTableSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
}

impl DeleteTableSyncData {
    pub fn new(db_table_data: &DbTableData, event_src: EventSource, persist: bool) -> Self {
        Self {
            table_data: SyncTableData::new(db_table_data, persist),
            event_src,
        }
    }
}
