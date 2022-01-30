use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_keys: Vec<String>,
    event_src: Option<EventSource>,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut sync_data = if let Some(event_src) = event_src {
        Some(InitPartitionsSyncData::new(
            &table_write_access,
            event_src,
            db_table.attributes.get_persist(),
        ))
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
