use std::sync::Arc;

use crate::db::DbTable;

use super::{
    states::{DeleteEventState, InitTableEventState, UpdatePartitionsState, UpdateRowsSyncState},
    SyncAttributes,
};

pub enum SyncEvent {
    UpdateTableAttributes {
        table: Arc<DbTable>,
        attr: SyncAttributes,
        table_is_just_created: bool,
        persist: bool,
        max_partitions_amount: Option<usize>,
    },
    InitTable(InitTableEventState),

    InitPartitions(UpdatePartitionsState),

    UpdateRows(UpdateRowsSyncState),

    Delete(DeleteEventState),

    DeleteTable {
        table: Arc<DbTable>,
        attr: SyncAttributes,
    },
}

impl SyncEvent {
    pub fn get_table(&self) -> Arc<DbTable> {
        match self {
            SyncEvent::UpdateTableAttributes {
                table,
                attr: _,
                table_is_just_created: _,
                persist: _,
                max_partitions_amount: _,
            } => table.clone(),
            SyncEvent::InitTable(state) => state.table.clone(),
            SyncEvent::InitPartitions(state) => state.table.clone(),
            SyncEvent::UpdateRows(state) => state.table.clone(),
            SyncEvent::Delete(state) => state.table.clone(),
            SyncEvent::DeleteTable { table, attr: _ } => table.clone(),
        }
    }
}
