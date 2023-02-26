use std::collections::HashMap;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn bulk_delete(
    app: &AppContext,
    db_table: &DbTableWrapper,
    rows_to_delete: HashMap<String, Vec<String>>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let mut sync_data = DeleteRowsEventSyncData::new(&table_data, event_src);

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result =
            table_data.bulk_remove_rows(&partition_key, row_keys.iter(), true);

        if let Some(removed_rows_result) = removed_rows_result {
            app.persist_markers
                .persist_partition(
                    table_data.name.as_str(),
                    partition_key.as_str(),
                    persist_moment,
                )
                .await;

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
