use crate::{
    db::{DbTableAttributesSnapshot, DbTableData, DbTableSnapshot},
    db_sync::SyncAttributes,
};

use super::SyncTableData;

pub struct InitTableEventSyncData {
    pub table_data: SyncTableData,
    pub attrs: SyncAttributes,
    pub table_snapshot: DbTableSnapshot,
}

impl InitTableEventSyncData {
    pub fn new(
        table_data: &DbTableData,
        table_attrs: DbTableAttributesSnapshot,
        attrs: SyncAttributes,
    ) -> Self {
        Self {
            table_data: SyncTableData::new(table_data, table_attrs.persist),
            attrs,
            table_snapshot: DbTableSnapshot::new(table_data, table_attrs),
        }
    }

    pub fn as_raw_bytes(&self) -> Vec<u8> {
        return self.table_snapshot.as_raw_bytes();
    }
}
