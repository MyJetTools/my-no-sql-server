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
    let mut table_write_access = db_table.data.write().await;

    let mut sync_data = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(db_table.as_ref(), attr))
    } else {
        None
    };

    let remove_row_result = {
        let db_partition = table_write_access.partitions.get_mut(partition_key);

        if db_partition.is_none() {
            return None;
        }

        let db_partition = db_partition.unwrap();

        let remove_result = super::db_actions::remove_db_row(
            app,
            db_table.name.as_str(),
            partition_key,
            db_partition,
            row_key,
            now,
            sync_data.as_mut(),
        )
        .await;

        if remove_result.is_none() {
            return None;
        }

        remove_result.unwrap()
    };

    if remove_row_result.partition_is_empty {
        table_write_access.partitions.remove(partition_key);
    }

    if let Some(sync_data) = sync_data {
        app.events_dispatcher
            .dispatch(SyncEvent::DeleteRows(sync_data))
            .await
    }

    return Some(remove_row_result.removed_row);
}
