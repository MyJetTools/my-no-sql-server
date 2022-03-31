use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    db::{DbTable, UpdatePartitionReadMoment},
    db_operations::DbOperationError,
};

use super::super::ReadOperationResult;

pub async fn get_single(
    table: &DbTable,
    partition_key: &str,
    row_key: &str,
) -> Result<ReadOperationResult, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let partition = table_data.get_partition(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(
        row_key,
        UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
    );

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}

pub async fn get_single_and_update_expiration_time(
    table: &DbTable,
    partition_key: &str,
    row_key: &str,
    expiration_time: Option<DateTimeAsMicroseconds>,
) -> Result<ReadOperationResult, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let partition = table_data.get_partition_mut(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row_and_update_expiration_time(
        row_key,
        UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
        expiration_time,
    );

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}
