use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_core::db::DbRow;
use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::InitPartitionsSyncData, EventSource, SyncEvent},
};

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTableWrapper>,
    partition_key: &str,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(partition_key);

    for (partition_key, db_rows) in entities {
        table_data.bulk_insert_or_replace(partition_key.as_str(), &db_rows);
    }

    app.persist_markers
        .persist_partition(table_data.name.as_str(), partition_key, persist_moment)
        .await;

    let state =
        InitPartitionsSyncData::new_as_update_partition(&table_data, partition_key, event_src);

    crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(state));

    Ok(())
}
