use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbTableSingleThreaded, DbTableWrapper},
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

//TODO - Use Method from TableData
pub async fn keep_max_partitions_amount(
    app: &AppContext,
    db_table_wrapper: &Arc<DbTableWrapper>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let partitions_amount = db_table_wrapper.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return Ok(());
    }

    let mut write_access = db_table_wrapper.data.write().await;

    gc_partitions(
        app,
        &mut write_access,
        event_src,
        max_partitions_amount,
        persist_moment,
    )
    .await?;

    Ok(())
}

pub async fn gc_partitions(
    app: &AppContext,
    write_access: &mut DbTableSingleThreaded,
    event_src: EventSource,
    max_partitions_amount: usize,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    let mut sync_state = InitPartitionsSyncData::new(&write_access.db_table, event_src);

    let gced_partitions_result = write_access
        .db_table
        .gc_and_keep_max_partitions_amount(max_partitions_amount);

    if let Some(gced_partitions) = gced_partitions_result {
        for (partition_key, _) in gced_partitions {
            write_access
                .persist_markers
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_ref(), persist_moment);

            sync_state.add(partition_key, None);
        }

        crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(sync_state));
    }

    Ok(())
}
