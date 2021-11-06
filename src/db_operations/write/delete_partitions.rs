use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::DeleteRowsEventSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_keys: Vec<String>,
    attr: Option<SyncAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut sync = if let Some(attr) = attr {
        Some(DeleteRowsEventSyncData::new(db_table.as_ref(), attr))
    } else {
        None
    };

    for partition_key in partition_keys {
        let remove_partition_result = table_write_access.partitions.remove(&partition_key);

        if let Some(removed_partition) = remove_partition_result {
            if let Some(state) = &mut sync {
                state.new_deleted_partition(partition_key, removed_partition);
            }
        }
    }

    if let Some(state) = sync {
        app.events_dispatcher
            .dispatch(SyncEvent::Delete(state))
            .await
    }
}
