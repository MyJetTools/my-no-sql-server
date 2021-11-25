use std::sync::Arc;

use crate::{
    app::AppContext,
    db::{DbRow, DbTable},
    db_json_entity::JsonTimeStamp,
    db_operations::DbOperationError,
    db_sync::{states::UpdateRowsSyncData, SyncAttributes, SyncEvent},
};

use serde::{Deserialize, Serialize};

#[inline]
pub async fn validate_before(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &str,
    entity_timestamp: Option<&str>,
) -> Result<(), DbOperationError> {
    if entity_timestamp.is_none() {
        return Err(DbOperationError::TimeStampFieldRequires);
    }

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_partition.unwrap().get_row(row_key);

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
    attr: Option<SyncAttributes>,
    entity_timestamp: &str,
    now: &JsonTimeStamp,
) -> Result<Arc<DbRow>, DbOperationError> {
    let mut table_data = db_table.data.write().await;

    let removed_row = {
        let db_partition = table_data.get_partition_mut(partition_key);

        if db_partition.is_none() {
            return Err(DbOperationError::RecordNotFound);
        }

        let db_partition = db_partition.unwrap();

        let current_db_row = db_partition.get_row(db_row.row_key.as_str());

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
        let (removed_row, _) = table_data
            .remove_row(partition_key, &db_row.row_key, false, now)
            .unwrap();

        removed_row
    };

    table_data.insert_row(&db_row, now);

    if let Some(attr) = attr {
        let mut update_rows_state =
            UpdateRowsSyncData::new(&table_data, db_table.attributes.get_persist(), attr);

        update_rows_state.add_row(db_row);

        app.events_dispatcher
            .dispatch(SyncEvent::UpdateRows(update_rows_state))
            .await
    }

    Ok(removed_row)
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
