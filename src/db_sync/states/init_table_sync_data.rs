use crate::{
    db::{DbTableData, DbTableSnapshot},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub attrs: SyncAttributes,
    pub table_snapshot: DbTableSnapshot,
}

impl InitTableEventSyncData {
    pub fn new(table_data: &DbTableData, attrs: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(table_data),
            attrs,
            table_snapshot: DbTableSnapshot::new(table_data),
        }
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        return self.table_snapshot.as_raw_bytes();
    }
}
