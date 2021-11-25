use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    row_key: &str,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Option<Arc<DbRow>> {
    let mut table_data = db_table.data.write().await;

    let sync_data = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(
            &table_data,
            db_table.attributes.get_persist(),
            attr,
        ))
    } else {
        None
    };

    let (removed_row, partition_is_empty) =
        table_data.remove_row(partition_key, row_key, true, now)?;

    if let Some(mut sync_data) = sync_data {
        if partition_is_empty {
            sync_data.new_deleted_partition(partition_key.to_string());
        } else {
            sync_data.add_deleted_row(partition_key, removed_row.clone())
        }

        app.events_dispatcher
            .dispatch(SyncEvent::DeleteRows(sync_data))
            .await
    }

    return Some(removed_row);
}
