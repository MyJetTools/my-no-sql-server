use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn delete_partitions(
    app: &AppContext,
    db_table: &DbTable,
    partition_keys: Vec<String>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) {
    let mut table_write_access = db_table.data.write().await;

    let mut sync_data = InitPartitionsSyncData::new(
        &table_write_access,
        event_src,
        db_table.attributes.get_persist(),
    );

    for partition_key in partition_keys {
        let remove_partition_result = table_write_access.remove_partition(&partition_key);

        if remove_partition_result.is_some() {
            table_write_access
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_str(), persist_moment);

            sync_data.add(partition_key, None);
        }
    }

    app.events_dispatcher
        .dispatch(SyncEvent::InitPartitions(sync_data));
}
