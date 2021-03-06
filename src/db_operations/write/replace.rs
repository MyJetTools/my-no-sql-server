use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable, UpdatePartitionReadMoment},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use super::WriteOperationResult;

#[inline]
pub async fn validate_before(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    row_key: &str,
    entity_timestamp: Option<&str>,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    if entity_timestamp.is_none() {
        return Err(DbOperationError::TimeStampFieldRequires);
    }

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_partition
        .unwrap()
        .get_row(row_key, UpdatePartitionReadMoment::None);

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    if db_row.unwrap().time_stamp != entity_timestamp.unwrap() {
        return Err(DbOperationError::OptimisticConcurencyUpdateFails);
    }

    Ok(())
}

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    partition_key: &str,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    entity_timestamp: &str,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let remove_result = {
        let db_partition = table_data.get_partition_mut(partition_key);

        if db_partition.is_none() {
            return Err(DbOperationError::RecordNotFound);
        }

        let db_partition = db_partition.unwrap();

        let current_db_row =
            db_partition.get_row(db_row.row_key.as_str(), UpdatePartitionReadMoment::None);

        match current_db_row {
            Some(current_db_row) => {
                if current_db_row.time_stamp != entity_timestamp {
                    return Err(DbOperationError::OptimisticConcurencyUpdateFails);
                }
            }
            None => {
                return Err(DbOperationError::RecordNotFound);
            }
        }
        let removed_result = table_data.remove_row(partition_key, &db_row.row_key, false, now);

        if removed_result.is_none() {
            None
        } else {
            Some(removed_result.unwrap().0)
        }
    };

    table_data.insert_row(&db_row, now);

    table_data
        .data_to_persist
        .mark_partition_to_persist(db_row.partition_key.as_str(), persist_moment);

    let mut update_rows_state =
        UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), event_src);

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(
        app,
        db_table.into(),
        SyncEvent::UpdateRows(update_rows_state),
    );

    match remove_result {
        Some(db_row) => Ok(WriteOperationResult::SingleRow(db_row)),
        None => Ok(WriteOperationResult::Empty),
    }
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
