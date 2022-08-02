use std::sync::Arc;

use my_no_sql_core::db_json_entity::JsonTimeStamp;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: Arc<DbTableWrapper>,
    partition_key: &str,
    row_key: &str,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;
    let mut write_access = db_table_wrapper.data.write().await;

    let remove_row_result = write_access
        .db_table
        .remove_row(partition_key, row_key, true, now);

    if remove_row_result.is_none() {
        return Ok(WriteOperationResult::Empty);
    }

    let (removed_row, partition_is_empty) = remove_row_result.unwrap();

    let mut sync_data = DeleteRowsEventSyncData::new(&write_access.db_table, event_src);

    write_access
        .persist_markers
        .data_to_persist
        .mark_row_to_persit(partition_key, row_key, persist_moment);

    if partition_is_empty {
        sync_data.new_deleted_partition(partition_key.to_string());
    } else {
        sync_data.add_deleted_row(partition_key, removed_row.clone())
    }

    crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));

    let result = WriteOperationResult::SingleRow(removed_row).into();
    Ok(result)
}
