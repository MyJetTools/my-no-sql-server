use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn bulk_delete(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    rows_to_delete: HashMap<String, Vec<String>>,
    event_src: EventSource,
    now: DateTimeAsMicroseconds,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;

    let mut sync_data = DeleteRowsEventSyncData::new(&write_access.db_table, event_src);

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result = write_access.db_table.bulk_remove_rows(
            partition_key.as_str(),
            row_keys.iter(),
            true,
            now,
        );

        if let Some(removed_rows_result) = removed_rows_result {
            write_access
                .persist_markers
                .data_to_persist
                .mark_row_keys_to_persit(
                    partition_key.as_str(),
                    row_keys.as_slice(),
                    persist_moment,
                );

            if removed_rows_result.1 {
                sync_data.new_deleted_partition(partition_key);
            } else {
                sync_data.add_deleted_rows(partition_key.as_str(), &removed_rows_result.0);
            }
        }
    }

    crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));

    Ok(())
}
