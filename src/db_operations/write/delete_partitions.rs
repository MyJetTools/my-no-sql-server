use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn delete_partitions(
    app: &AppContext,
    db_table: &DbTableWrapper,
    partition_keys: Vec<String>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut write_access = db_table.data.write().await;

    let mut sync_data = InitPartitionsSyncData::new(&write_access.db_table, event_src);

    for partition_key in partition_keys {
        let remove_partition_result = write_access.db_table.remove_partition(&partition_key);

        if remove_partition_result.is_some() {
            write_access
                .persist_markers
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_str(), persist_moment);

            sync_data.add(partition_key, None);
        }
    }

    crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(sync_data));

    Ok(())
}
