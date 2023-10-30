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

    let result = table_data.insert_or_replace_row(&db_row, Some(now));

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);

    app.persist_markers
        .persist_partition(
            db_table.name.as_str(),
            db_row.partition_key.as_ref(),
            persist_moment,
        )
        .await;

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    let result = match result {
        Some(db_row) => WriteOperationResult::SingleRow(db_row),
        None => WriteOperationResult::Empty,
    };

    Ok(result)
}
