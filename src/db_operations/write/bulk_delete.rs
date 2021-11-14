use std::collections::HashMap;

use crate::{
    app::AppContext,
    db::DbTable,
    db_json_entity::JsonTimeStamp,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    rows_to_delete: HashMap<String, Vec<String>>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) {
    let mut table_data = db_table.data.write().await;

    let mut sync_data = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(&table_data, attr))
    } else {
        None
    };

    for (partition_key, row_keys) in rows_to_delete {
        let removed_rows_result = table_data.bulk_remove_rows(
            partition_key.as_str(),
            row_keys.iter().map(|itm| itm.as_str()),
            true,
            now,
        );
        if let Some(sync_data) = &mut sync_data {
            if let Some(removed_rows_result) = removed_rows_result {
                if removed_rows_result.1 {
                    sync_data.add_deleted_rows(partition_key.as_str(), &removed_rows_result.0);
                } else {
                    sync_data.add_deleted_rows(partition_key.as_str(), &removed_rows_result.0);
                }
            }
        }
    }

    if let Some(sync_data) = sync_data {
        app.events_dispatcher
            .dispatch(SyncEvent::DeleteRows(sync_data))
            .await;
    }
}
