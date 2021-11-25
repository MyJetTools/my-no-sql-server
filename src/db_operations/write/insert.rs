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

    let partition = read_access.get_partition(partition_key);

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
    db_row: Arc<DbRow>,
    attr: Option<SyncAttributes>,
    now: &JsonTimeStamp,
) -> Result<(), DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let inserted = table_data.insert_row(&db_row, now);

    if !inserted {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    if let Some(attr) = attr {
        let mut update_rows_state =
            UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), attr);

        update_rows_state.add_row(db_row);

        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(update_rows_state))
            .await;
    }
    return Ok(());
}
