use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    row_key: &str,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    let remove_row_result = table_data.remove_row(partition_key, row_key, true, now);

    if remove_row_result.is_none() {
        return Ok(WriteOperationResult::Empty);
    }

    let (removed_row, partition_is_empty) = remove_row_result.unwrap();

    let mut sync_data =
        DeleteRowsEventSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    table_data
        .data_to_persist
        .mark_partition_to_persist(partition_key, persist_moment);

    if partition_is_empty {
        sync_data.new_deleted_partition(partition_key.to_string());
    } else {
        sync_data.add_deleted_row(partition_key, removed_row.clone())
    }

    app.events_dispatcher
        .dispatch(SyncEvent::DeleteRows(sync_data));

    let result = WriteOperationResult::SingleRow(removed_row).into();
    Ok(result)
}
