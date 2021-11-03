use crate::{db::DbTable, db_sync::SyncAttributes};

use super::SyncTableData;

pub struct DeleteTableSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
}

impl DeleteTableSyncData {
    pub fn new(db_table: &DbTable, attr: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(db_table),
            attr,
        }
    }
}
