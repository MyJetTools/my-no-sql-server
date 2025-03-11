use my_no_sql_sdk::core::db::{PartitionKeyParameter, RowKeyParameter};
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::server::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn bulk_delete(
    app: &AppContext,
    db_table: &DbTableWrapper,
    rows_to_delete: impl Iterator<Item = (impl PartitionKeyParameter, Vec<impl RowKeyParameter>)>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let mut sync_data = DeleteRowsEventSyncData::new(&table_data, event_src);

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result =
            table_data.bulk_remove_rows(&partition_key, row_keys.into_iter(), true, Some(now));

        if let Some((partition_key, removed_rows, partition_is_empty)) = removed_rows_result {
            if partition_is_empty {
                sync_data.new_deleted_partition(&partition_key);

                app.persist_markers
                    .persist_partition(&table_data.name, &partition_key, persist_moment)
                    .await;
            } else {
                sync_data.add_deleted_rows(&partition_key, &removed_rows);

                app.persist_markers
                    .delete_db_rows(
                        &table_data.name,
                        &partition_key,
                        persist_moment,
                        removed_rows.iter(),
                    )
                    .await;
            }
        }
    }

    crate::operations::sync::dispatch(app, SyncEvent::DeleteRows(sync_data));

    Ok(())
}
