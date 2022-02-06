use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use super::WriteOperationResult;

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> WriteOperationResult {
    let mut table_data = db_table.data.write().await;

    let result = table_data.insert_or_replace_row(&db_row, now);

    let mut update_rows_state =
        UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    table_data
        .data_to_persist
        .mark_partition_to_persist(db_row.partition_key.as_ref(), persist_moment);

    update_rows_state.rows_by_partition.add_row(db_row);
    app.events_dispatcher
        .dispatch(SyncEvent::UpdateRows(update_rows_state));

    match result {
        Some(db_row) => WriteOperationResult::SingleRow(db_row),
        None => WriteOperationResult::Empty,
    }
}
