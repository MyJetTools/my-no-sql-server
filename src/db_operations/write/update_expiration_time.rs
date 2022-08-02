use my_no_sql_core::db::{UpdateExpirationTimeModel, UpdatePartitionReadMoment};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn update_expiration_time(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    row_keys: &[String],
    update_expiration_time: &UpdateExpirationTimeModel,
    event_src: EventSource,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let now = DateTimeAsMicroseconds::now();
    let mut write_access = db_table_wrapper.data.write().await;

    let mut update_sync_data = UpdateRowsSyncData::new(&write_access.db_table, event_src);

    let db_partition = write_access.db_table.get_partition_mut(partition_key);

    if db_partition.is_none() {
        return Ok(());
    }

    let db_partition = db_partition.unwrap();

    for row_key in row_keys {
        let updated_row = db_partition.get_row_and_update_expiration_time(
            row_key.as_str(),
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
            update_expiration_time,
        );

        if let Some(db_row) = updated_row {
            update_sync_data.rows_by_partition.add_row(db_row);
        }
    }

    if update_sync_data.rows_by_partition.has_elements() {
        crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_sync_data));
    }

    Ok(())
}
