use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn bulk_delete(
    app: &AppContext,
    db_table: &DbTable,
    rows_to_delete: HashMap<String, Vec<String>>,
    event_src: EventSource,
    now: DateTimeAsMicroseconds,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let mut sync_data =
        DeleteRowsEventSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result =
            table_data.bulk_remove_rows(partition_key.as_str(), row_keys.iter(), true, now);

        if let Some(removed_rows_result) = removed_rows_result {
            table_data
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_str(), persist_moment);

            if removed_rows_result.1 {
                sync_data.new_deleted_partition(partition_key);
            } else {
                sync_data.add_deleted_rows(partition_key.as_str(), &removed_rows_result.0);
            }
        }
    }

    crate::operations::sync::dispatch(app, Some(db_table), SyncEvent::DeleteRows(sync_data));

    Ok(())
}
