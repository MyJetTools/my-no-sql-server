use std::sync::Arc;

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn validate_before(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    partition_key: &str,
    row_key: &str,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let read_access = db_table.data.read().await;

    let partition = read_access.get_partition(partition_key);

    if partition.is_none() {
        return Ok(());
    }

    let partition = partition.unwrap();

    if partition.get_row(row_key).is_some() {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    Ok(())
}

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTableWrapper>,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let inserted = table_data.insert_row(&db_row, Some(now));

    if !inserted {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    app.persist_markers
        .persist_partition(&table_data, db_row.get_partition_key(), persist_moment)
        .await;

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    return Ok(());
}
