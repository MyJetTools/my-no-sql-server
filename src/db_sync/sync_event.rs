use super::states::{
    DeleteRowsEventSyncData, DeleteTableSyncData, InitPartitionsSyncData, InitTableEventSyncData,
    UpdateRowsSyncData, UpdateTableAttributesSyncData,
};

pub enum SyncEvent {
    UpdateTableAttributes(UpdateTableAttributesSyncData),

    InitTable(InitTableEventSyncData),

    InitPartitions(InitPartitionsSyncData),

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

    pub fn has_elements_to_dispatch(&self) -> bool {
        match self {
            SyncEvent::UpdateTableAttributes(_) => true,
            SyncEvent::InitTable(_) => true,
            SyncEvent::InitPartitions(data) => data.partitions_to_update.len() > 0,
            SyncEvent::UpdateRows(data) => data.updated_rows_by_partition.len() > 0,
            SyncEvent::DeleteRows(data) => {
                data.deleted_partitions.is_some() || data.deleted_rows.is_some()
            }
            SyncEvent::DeleteTable(_) => true,
        }
    }
}
