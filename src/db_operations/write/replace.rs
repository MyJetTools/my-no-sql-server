use std::sync::Arc;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, EventSource, SyncEvent},
};

use my_no_sql_core::{
    db::{DbRow, UpdatePartitionReadMoment},
    db_json_entity::JsonTimeStamp,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use super::WriteOperationResult;

#[inline]
pub async fn validate_before(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    row_key: &str,
    entity_timestamp: Option<&str>,
) -> Result<(), DbOperationError> {
    super::super::check_app_states(app)?;

    if entity_timestamp.is_none() {
        return Err(DbOperationError::TimeStampFieldRequires);
    }

    let read_access = db_table_wrapper.data.read().await;

    let db_partition = read_access.db_table.get_partition(partition_key);

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
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    db_row: Arc<DbRow>,
    event_src: EventSource,
    entity_timestamp: &str,
    now: &JsonTimeStamp,
    persist_moment: DateTimeAsMicroseconds,
) -> Result<WriteOperationResult, DbOperationError> {
    let mut write_access = db_table_wrapper.data.write().await;

    let remove_result = {
        let db_partition = write_access.db_table.get_partition_mut(partition_key);

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
        let removed_result =
            write_access
                .db_table
                .remove_row(partition_key, &db_row.row_key, false, now);

        if removed_result.is_none() {
            None
        } else {
            Some(removed_result.unwrap().0)
        }
    };

    write_access.db_table.insert_row(&db_row, now);

    write_access
        .persist_markers
        .data_to_persist
        .mark_row_to_persit(
            db_row.partition_key.as_str(),
            db_row.row_key.as_ref(),
            persist_moment,
        );

    let mut update_rows_state = UpdateRowsSyncData::new(&write_access.db_table, event_src);

    update_rows_state.rows_by_partition.add_row(db_row);

    crate::operations::sync::dispatch(app, SyncEvent::UpdateRows(update_rows_state));

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
