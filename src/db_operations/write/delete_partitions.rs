use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitPartitionsSyncData, SyncAttributes, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_keys: Vec<String>,
    attr: Option<SyncAttributes>,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut sync_data = if let Some(attr) = attr {
        Some(InitPartitionsSyncData::new(&table_write_access, attr))
    } else {
        None
    };

    for partition_key in partition_keys {
        let remove_partition_result = table_write_access.remove_partition(&partition_key);

        if remove_partition_result.is_some() {
            if let Some(sync_data) = &mut sync_data {
                sync_data.add(partition_key, None);
            }
        }
    }

    if let Some(sync_data) = sync_data {
        app.events_dispatcher
            .dispatch(SyncEvent::InitPartitions(sync_data))
            .await
    }
}
