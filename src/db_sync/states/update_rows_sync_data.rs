use crate::{
    db::{db_snapshots::DbRowsByPartitionsSnapshot, DbTableData},
    db_sync::EventSource,
};

use super::SyncTableData;

pub struct UpdateRowsSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub rows_by_partition: DbRowsByPartitionsSnapshot,
}

impl UpdateRowsSyncData {
    pub fn new(table_data: &DbTableData, persist: bool, event_src: EventSource) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, persist),
            event_src,
            rows_by_partition: DbRowsByPartitionsSnapshot::new(),
        }
    }
}
