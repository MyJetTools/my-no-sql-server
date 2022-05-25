use crate::{
    db::{db_snapshots::DbTableSnapshot, DbTable, DbTableAttributesSnapshot, DbTableData},
    db_sync::EventSource,
};

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub event_src: EventSource,
    pub table_snapshot: DbTableSnapshot,
}

impl InitTableEventSyncData {
    pub fn new(
        db_table: &DbTable,
        table_data: &DbTableData,
        table_attrs: DbTableAttributesSnapshot,
        event_src: EventSource,
    ) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, table_attrs.persist),
            event_src,
            table_snapshot: DbTableSnapshot::new(
                db_table.get_last_update_time(),
                table_data,
                table_attrs,
            ),
        }
    }
}
