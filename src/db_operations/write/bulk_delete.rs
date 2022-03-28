use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_json_entity::JsonTimeStamp,
    db_sync::{states::DeleteRowsEventSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    rows_to_delete: HashMap<String, Vec<String>>,
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut table_data = db_table.data.write().await;

    let mut sync_data =
        DeleteRowsEventSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result =
            table_data.bulk_remove_rows(partition_key.as_str(), row_keys.iter(), true, now);

        if let Some(removed_rows_result) = removed_rows_result {
            table_data
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_str(), persist_moment);

            if removed_rows_result.1 {
                sync_data.new_deleted_partition(partition_key);
            } else {
                sync_data.add_deleted_rows(partition_key.as_str(), &removed_rows_result.0);
            }
        }
    }

    app.events_dispatcher
        .dispatch(SyncEvent::DeleteRows(sync_data));
}
