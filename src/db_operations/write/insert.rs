use std::sync::Arc;

use my_no_sql_core::{
    db::{DbRow, UpdatePartitionReadMoment},
    db_json_entity::JsonTimeStamp,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn validate_before(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    row_key: &str,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let read_access = db_table_wrapper.data.read().await;

    let partition = read_access.db_table.get_partition(partition_key);

    if partition.is_none() {
        return Ok(());
    }

    let partition = partition.unwrap();

    if partition
        .get_row(row_key, UpdatePartitionReadMoment::None)
        .is_some()
    {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    Ok(())
}

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    let mut write_access = db_table_wrapper.data.write().await;

    let inserted = write_access.db_table.insert_row(&db_row, now);

    if !inserted {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    write_access
        .persist_markers
        .data_to_persist
        .mark_row_to_persit(
            db_row.partition_key.as_str(),
            db_row.row_key.as_ref(),
            persist_moment,
        );

    let mut update_rows_state = UpdateRowsSyncData::new(&write_access.db_table, event_src);

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    return Ok(());
}
