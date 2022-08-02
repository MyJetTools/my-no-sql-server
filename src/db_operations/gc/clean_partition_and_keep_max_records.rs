use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    max_rows_amount: usize,
    event_source: EventSource,
    sync_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;

    let partition = write_access.db_table.get_partition_mut(partition_key);

    if partition.is_none() {
        return Ok(());
    }

    let partition = partition.unwrap();

    let gced_rows_result = partition.gc_rows(max_rows_amount);

    if let Some(gced_rows) = gced_rows_result {
        write_access
            .persist_markers
            .data_to_persist
            .mark_partition_to_persist(partition_key, sync_moment);

        let mut sync_data = DeleteRowsEventSyncData::new(&write_access.db_table, event_source);

        sync_data.add_deleted_rows(partition_key, &gced_rows);
        crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));
    }

    Ok(())
}
