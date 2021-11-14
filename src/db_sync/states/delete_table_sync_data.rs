use crate::{db::DbTableData, db_sync::SyncAttributes};

use super::SyncTableData;

pub struct DeleteTableSyncData {
    pub table_data: SyncTableData,
    pub attr: SyncAttributes,
}

impl DeleteTableSyncData {
    pub fn new(db_table_data: &DbTableData, attr: SyncAttributes) -> Self {
        Self {
            table_data: SyncTableData::new(db_table_data),
            attr,
        }
    }
}
