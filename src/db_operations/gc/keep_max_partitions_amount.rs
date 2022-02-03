use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    max_partitions_amount: usize,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) {
    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return;
    }

    let mut table_data = db_table.data.write().await;

    let mut sync_state =
        InitPartitionsSyncData::new(&table_data, event_src, db_table.attributes.get_persist());

    let gced_partitions_result =
        table_data.gc_and_keep_max_partitions_amount(max_partitions_amount);

    if let Some(gced_partitions) = gced_partitions_result {
        for (partition_key, _) in gced_partitions {
            table_data
                .data_to_persist
                .mark_partition_to_persist(partition_key.as_ref(), persist_moment);

            sync_state.add(partition_key, None);
        }
        app.events_dispatcher
            .dispatch(SyncEvent::InitPartitions(sync_state));
    }
}
