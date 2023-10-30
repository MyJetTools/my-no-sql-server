use std::{collections::BTreeMap, sync::Arc};

use my_no_sql_sdk::core::db::DbRow;
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
    partition_key: &String,
    entities: BTreeMap<String, Vec<Arc<DbRow>>>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;
    let mut table_data = db_table.data.write().await;

    table_data.remove_partition(partition_key, None);

    for (partition_key, db_rows) in entities {
        println!("Inserting to partition: {}", partition_key);

        for db_row in &db_rows {
            println!("Inserting row: {}/{}", db_row.partition_key, db_row.row_key);
        }
        table_data.bulk_insert_or_replace(&partition_key, &db_rows, Some(now));
    }

    app.persist_markers
        .persist_partition(table_data.name.as_str(), partition_key, persist_moment)
        .await;

    let state =
        InitPartitionsSyncData::new_as_update_partition(&table_data, partition_key, event_src);

    crate::operations::sync::dispatch(app, SyncEvent::InitPartitions(state));

    Ok(())
}
