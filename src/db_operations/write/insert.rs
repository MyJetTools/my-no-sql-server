use std::sync::Arc;

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::core::{db::DbRow, db_json_entity::DbJsonEntityWithContent};
use my_no_sql_sdk::server::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

pub async fn validate_before(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    db_entity: DbJsonEntityWithContent<'_>,
) -> Result<DbRow, DbOperationError> {
    super::super::check_app_states(app)?;
    let read_access = db_table.data.read().await;

    let partition = read_access.get_partition(db_entity.get_partition_key());

    if partition.is_none() {
        return Ok(db_entity.into_db_row()?);
    }

    let partition = partition.unwrap();

    if partition.get_row(db_entity.get_row_key()).is_some() {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    Ok(db_entity.into_db_row()?)
}

pub async fn execute(
    app: &AppContext,
    db_table: Arc<DbTableWrapper>,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    persist_moment: DateTimeAsMicroseconds,
    now: DateTimeAsMicroseconds,
) -> Result<(), DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let partition_key = table_data.insert_row(&db_row, Some(now));

    if partition_key.is_none() {
        return Err(DbOperationError::RecordAlreadyExists);
    }

    let partition_key = partition_key.unwrap();

    app.persist_markers
        .persist_rows(
            &table_data.name,
            &partition_key,
            persist_moment,
            [&db_row].into_iter(),
        )
        .await;

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);

    update_rows_state
        .rows_by_partition
        .add_row(partition_key, db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    return Ok(());
}
