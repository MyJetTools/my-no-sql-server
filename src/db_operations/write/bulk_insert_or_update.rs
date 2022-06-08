use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut table_data = db_table.data.write().await;

    let mut update_rows_state =
        UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    let mut has_insert_or_replace = false;

    for (partition_key, db_rows) in rows_by_partition {
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, now);
        has_insert_or_replace = true;

        update_rows_state
            .rows_by_partition
            .add_rows(partition_key.as_str(), db_rows);

        table_data
            .data_to_persist
            .mark_partition_to_persist(partition_key.as_str(), persist_moment);
    }

    if has_insert_or_replace {
        db_table.set_last_update_time(DateTimeAsMicroseconds::now());

        app.events_dispatcher.dispatch(
            db_table.as_ref().into(),
            SyncEvent::UpdateRows(update_rows_state),
        );
    }

    Ok(())
}
