use std::{collections::BTreeMap, sync::Arc};

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTable>,
    partition_key: &str,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: Option<EventSource>,
    now: &JsonTimeStamp,
) -> Result<(), DbOperationError> {
    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(partition_key);

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    if let Some(event_src) = event_src {
        let state = InitPartitionsSyncData::new_as_update_partition(
            &table_data,
            partition_key,
            event_src,
            db_table.attributes.get_persist(),
        );

        app.events_dispatcher
            .dispatch(SyncEvent::InitPartitions(state))
            .await;
    }

    Ok(())
}
