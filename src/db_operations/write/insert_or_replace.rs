use std::sync::Arc;

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTableWrapper>,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let (partition_key, _) = table_data.insert_or_replace_row(&db_row, Some(now));

    app.persist_markers
        .persist_partition(&table_data, &partition_key, persist_moment)
        .await;

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);
    update_rows_state
        .rows_by_partition
        .add_row(partition_key, db_row.clone());

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    Ok(WriteOperationResult::SingleRow(db_row))
}
