use super::states::{
    DeleteRowsEventSyncData, DeleteTableSyncData, InitTableEventSyncData, UpdatePartitionsSyncData,
    UpdateRowsSyncData, UpdateTableAttributesSyncData,
};

pub enum SyncEvent {
    UpdateTableAttributes(UpdateTableAttributesSyncData),

    InitTable(InitTableEventSyncData),

    InitPartitions(UpdatePartitionsSyncData),

    UpdateRows(UpdateRowsSyncData),

    DeleteRows(DeleteRowsEventSyncData),

    DeleteTable(DeleteTableSyncData),
}

impl SyncEvent {
    pub fn get_table_name(&self) -> &str {
        match self {
            SyncEvent::UpdateTableAttributes(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitTable(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitPartitions(data) => data.table_data.table_name.as_ref(),
            SyncEvent::UpdateRows(data) => data.table_data.table_name.as_ref(),
            SyncEvent::DeleteRows(data) => data.table_data.table_name.as_ref(),
            SyncEvent::DeleteTable(data) => data.table_data.table_name.as_ref(),
        }
    }
}
