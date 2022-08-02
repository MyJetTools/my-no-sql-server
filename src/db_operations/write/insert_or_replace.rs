use std::sync::Arc;

use my_no_sql_core::{db::DbRow, db_json_entity::JsonTimeStamp};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: Arc<DbTableWrapper>,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;
    let mut write_access = db_table_wrapper.data.write().await;

    let result = write_access.db_table.insert_or_replace_row(&db_row, now);

    let mut update_rows_state = UpdateRowsSyncData::new(&write_access.db_table, event_src);

    write_access
        .persist_markers
        .data_to_persist
        .mark_row_to_persit(
            db_row.partition_key.as_str(),
            db_row.row_key.as_ref(),
            persist_moment,
        );

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    let result = match result {
        Some(db_row) => WriteOperationResult::SingleRow(db_row),
        None => WriteOperationResult::Empty,
    };

    Ok(result)
}
