use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, SyncAttributes, SyncEvent},
};

pub async fn validate_before(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &str,
) -> Result<(), DbOperationError> {
    let read_access = db_table.data.read().await;

    let partition = read_access.partitions.get(partition_key);

    if partition.is_none() {
        return Ok(());
    }

    let partition = partition.unwrap();

    if partition.get_row(row_key).is_some() {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    Ok(())
}

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    db_row: Arc<DbRow>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Result<(), DbOperationError> {
    let mut table_write_access = db_table.data.write().await;

    let db_partition = table_write_access.get_or_create_partition(partition_key);

    let inserted = db_partition.insert(db_row.clone(), Some(now));

    if !inserted {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    if let Some(attr) = attr {
        let mut update_rows_state = UpdateRowsSyncData::new(db_table, attr);

        update_rows_state.add_row(partition_key, db_row);

        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(update_rows_state))
            .await;
    }
    return Ok(());
}
