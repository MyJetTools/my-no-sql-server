use std::sync::Arc;

use crate::{
    app::AppContext,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use my_no_sql_sdk::core::rust_extensions::date_time::DateTimeAsMicroseconds;
use my_no_sql_sdk::core::{
    db::DbRow,
    db_json_entity::{DbJsonEntityWithContent, JsonTimeStamp},
};
use my_no_sql_server_core::DbTableWrapper;
use serde::{Deserialize, Serialize};

use super::WriteOperationResult;

#[inline]
pub async fn validate_before(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    db_entity: DbJsonEntityWithContent<'_>,
) -> Result<DbRow, DbOperationError> {
    super::super::check_app_states(app)?;

    if db_entity.get_time_stamp().is_none() {
        return Err(DbOperationError::TimeStampFieldRequires);
    }

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(db_entity.get_partition_key());

    if db_partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_partition.unwrap().get_row(db_entity.get_row_key());

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    if db_row.unwrap().get_time_stamp() != db_entity.get_time_stamp().unwrap() {
        return Err(DbOperationError::OptimisticConcurrencyUpdateFails);
    }

    Ok(db_entity.into_db_row()?)
}

pub async fn execute(
    app: &AppContext,
    db_table: &Arc<DbTableWrapper>,
    db_row: Arc<DbRow>,
    event_src: EventSource,

    persist_moment: DateTimeAsMicroseconds,
    now: &JsonTimeStamp,
) -> Result<WriteOperationResult, DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let partition_key = {
        let db_partition = table_data.get_partition_mut(db_row.get_partition_key());

        if db_partition.is_none() {
            return Err(DbOperationError::RecordNotFound);
        }

        let db_partition = db_partition.unwrap();

        let current_db_row = db_partition.get_row(db_row.get_row_key());

        match current_db_row {
            Some(current_db_row) => {
                if current_db_row.get_time_stamp() != db_row.get_time_stamp() {
                    return Err(DbOperationError::OptimisticConcurrencyUpdateFails);
                }
            }
            None => {
                return Err(DbOperationError::RecordNotFound);
            }
        }

        db_partition.partition_key.clone()
    };

    table_data.remove_row(&partition_key, &db_row, false, None);

    table_data.insert_row(&db_row, Some(now.date_time));

    app.persist_markers
        .persist_partition(&table_data, &db_row, persist_moment)
        .await;

    let mut update_rows_state = UpdateRowsSyncData::new(&table_data, event_src);
    update_rows_state
        .rows_by_partition
        .add_row(partition_key, db_row.clone());

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

    Ok(WriteOperationResult::SingleRow(db_row))
}

#[derive(Deserialize, Serialize)]
pub struct DeleteModel {
    pub key: String,
    pub values: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test() {
        let str = r###"{"Key1": ["Value1", "Value2"], "Key2": ["Value3", "Value4"]}"###;

        let result: HashMap<String, Vec<String>> = serde_json::from_slice(str.as_bytes()).unwrap();

        assert_eq!(2, result.len())
    }
}
