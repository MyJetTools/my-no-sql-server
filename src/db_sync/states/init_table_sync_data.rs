use crate::{
    db::{DbTableAttributesSnapshot, DbTableData, DbTableSnapshot},
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
        table_data: &DbTableData,
        table_attrs: DbTableAttributesSnapshot,
        event_src: EventSource,
    ) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, table_attrs.persist),
            event_src,
            table_snapshot: DbTableSnapshot::new(table_data, table_attrs),
        }
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        return self.table_snapshot.as_raw_bytes();
    }
}
