use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);

    let mut has_insert_or_replace = false;

    for (partition_key, db_rows) in rows_by_partition {
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, Some(now));
        has_insert_or_replace = true;

        update_rows_state
            .rows_by_partition
            .add_rows(partition_key.as_str(), db_rows);

        app.persist_markers
            .persist_partition(
                table_data.name.as_str(),
                partition_key.as_str(),
                persist_moment,
            )
            .await;
    }

    if has_insert_or_replace {
        crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));
    }

    Ok(())
}
