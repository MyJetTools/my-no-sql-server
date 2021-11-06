use crate::{
    db::{DbTable, DbTableSnapshot},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub attrs: SyncAttributes,
    pub table_snapshot: Option<DbTableSnapshot>,
}

impl InitTableEventSyncData {
    pub fn new(table: &DbTable, attrs: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(table),
            attrs,
            table_snapshot: None,
        }
    }

    pub fn add_table_snapshot(&mut self, snapshot: DbTableSnapshot) {
        self.table_snapshot = Some(snapshot);
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        return self.table_snapshot.as_ref().unwrap().as_raw_bytes();
    }
}
