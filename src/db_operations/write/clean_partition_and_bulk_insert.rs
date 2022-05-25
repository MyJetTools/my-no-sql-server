use std::{collections::BTreeMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;

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
    event_src: EventSource,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(partition_key);

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows, now);
    }

    table_data
        .data_to_persist
        .mark_partition_to_persist(partition_key, persist_moment);

    let state = InitPartitionsSyncData::new_as_update_partition(
        &table_data,
        partition_key,
        event_src,
        db_table.attributes.get_persist(),
    );

    app.events_dispatcher
        .dispatch(db_table.as_ref().into(), SyncEvent::InitPartitions(state));

    Ok(())
}
