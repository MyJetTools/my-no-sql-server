use std::sync::Arc;

use my_no_sql_sdk::core::db::DbRow;
use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncEventData, EventSource, SyncEvent},
};

pub async fn clean_partition_and_bulk_insert(
    app: &AppContext,
    db_table: Arc<DbTableWrapper>,
    partition_to_clean: String,
    entities: Vec<(String, Vec<Arc<DbRow>>)>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(&partition_to_clean, None);

    let mut partition_keys = Vec::new();

    for (partition_key, db_rows) in entities {
        let (partition_key, _) =
            table_data.bulk_insert_or_replace(&partition_key, &db_rows, Some(now));

        partition_keys.push(partition_key);
    }

    for partition_key in partition_keys {
        app.persist_markers
            .persist_partition(&table_data.name, &partition_key, persist_moment)
            .await;

        let state = InitPartitionsSyncEventData::new_as_update_partition(
            &table_data,
            partition_key.into(),
            event_src.clone(),
        );

        crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(state));
    }

    Ok(())
}
