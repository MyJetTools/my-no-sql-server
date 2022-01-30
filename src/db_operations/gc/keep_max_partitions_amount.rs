use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTable,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    max_partitions_amount: usize,
    event_src: Option<EventSource>,
) {
    let partitions_amount = db_table.get_partitions_amount().await;

    if partitions_amount <= max_partitions_amount {
        return;
    }

    let mut table_data = db_table.data.write().await;

    let sync = if let Some(event_src) = event_src {
        Some(InitPartitionsSyncData::new(
            &table_data,
            event_src,
            db_table.attributes.get_persist(),
        ))
    } else {
        None
    };

    let gced_partitions_result =
        table_data.gc_and_keep_max_partitions_amount(max_partitions_amount);

    if let Some(gced_partitions) = gced_partitions_result {
        if let Some(mut state) = sync {
            for (partition_key, _) in gced_partitions {
                state.add(partition_key, None);
            }
            app.events_dispatcher
                .dispatch(SyncEvent::InitPartitions(state))
                .await;
        }
    }
}
