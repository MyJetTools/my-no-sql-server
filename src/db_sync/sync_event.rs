use super::states::{
    DeleteEventSyncData, DeleteTableSyncData, InitTableEventSyncData, UpdatePartitionsSyncData,
    UpdateRowsSyncData, UpdateTableAttributesSyncData,
};

pub enum SyncEvent {
    UpdateTableAttributes(UpdateTableAttributesSyncData),

    InitTable(InitTableEventSyncData),

    InitPartitions(UpdatePartitionsSyncData),

    UpdateRows(UpdateRowsSyncData),

    Delete(DeleteEventSyncData),

    DeleteTable(DeleteTableSyncData),
}

impl SyncEvent {
    pub fn get_table_name(&self) -> &str {
        match self {
            SyncEvent::UpdateTableAttributes(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitTable(data) => data.table_data.table_name.as_ref(),
            SyncEvent::InitPartitions(data) => data.table_data.table_name.as_ref(),
            SyncEvent::UpdateRows(data) => data.table_data.table_name.as_ref(),
            SyncEvent::Delete(data) => data.table_data.table_name.as_ref(),
            SyncEvent::DeleteTable(data) => data.table_data.table_name.as_ref(),
        }
    }
}
