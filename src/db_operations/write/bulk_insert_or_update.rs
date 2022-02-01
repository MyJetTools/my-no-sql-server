use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    rows_by_partition: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut table_data = db_table.data.write().await;

    let mut update_rows_state =
        UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    for (partition_key, db_rows) in rows_by_partition {
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, now);

        update_rows_state.add_rows(partition_key.as_str(), db_rows);

        table_data
            .data_to_persist
            .mark_partition_to_persist(partition_key.as_str(), persist_moment);
    }

    app.events_dispatcher
        .dispatch(SyncEvent::UpdateRows(update_rows_state));
}
