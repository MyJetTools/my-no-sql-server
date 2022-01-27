use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_json_entity::JsonTimeStamp,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    row_key: &str,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> WriteOperationResult {
    let mut table_data = db_table.data.write().await;

    let sync_data = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(
            &table_data,
            db_table.attributes.get_persist(),
            attr,
        ))
    } else {
        None
    };

    let remove_row_result = table_data.remove_row(partition_key, row_key, true, now);

    if remove_row_result.is_none() {
        return WriteOperationResult::Empty;
    }

    let (removed_row, partition_is_empty) = remove_row_result.unwrap();

    if let Some(mut sync_data) = sync_data {
        if partition_is_empty {
            sync_data.new_deleted_partition(partition_key.to_string());
        } else {
            sync_data.add_deleted_row(partition_key, removed_row.clone())
        }

        app.events_dispatcher
            .dispatch(SyncEvent::DeleteRows(sync_data))
            .await
    }

    WriteOperationResult::SingleRow(removed_row).into()
}
