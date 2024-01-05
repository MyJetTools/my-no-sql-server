use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn delete_partitions<'s, TPartitions: Iterator<Item = &'s str>>(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    partition_keys: TPartitions,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_write_access = db_table.data.write().await;

    let mut sync_data = InitPartitionsSyncData::new(&table_write_access, event_src);

    for partition_key in partition_keys {
        let remove_partition_result =
            table_write_access.remove_partition(&partition_key, Some(now));

        if remove_partition_result.is_some() {
            app.persist_markers
                .persist_partition(&table_write_access, partition_key, persist_moment)
                .await;

            sync_data.add(partition_key.to_string(), None);
        }
    }

    crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(sync_data));

    Ok(())
}
