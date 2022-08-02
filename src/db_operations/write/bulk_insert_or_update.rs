use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::{db::DbRow, db_json_entity::JsonTimeStamp};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: Arc<DbTableWrapper>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut write_access = db_table_wrapper.data.write().await;

    let mut update_rows_state = UpdateRowsSyncData::new(&write_access.db_table, event_src);

    let mut has_insert_or_replace = false;

    for (partition_key, db_rows) in rows_by_partition {
        write_access
            .db_table
            .bulk_insert_or_replace(&partition_key, &db_rows, now);
        has_insert_or_replace = true;

        update_rows_state
            .rows_by_partition
            .add_rows(partition_key.as_str(), db_rows);
    }

    if has_insert_or_replace {
        for (parition_key, db_rows) in &update_rows_state.rows_by_partition.partitions {
            write_access
                .persist_markers
                .data_to_persist
                .mark_rows_to_persit(parition_key.as_str(), db_rows.as_slice(), persist_moment);
        }

        crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));
    }

    Ok(())
}
