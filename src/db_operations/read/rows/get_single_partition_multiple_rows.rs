use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::lazy::LazyVec;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

use super::super::ReadOperationResult;

pub async fn get_single_partition_multiple_rows(
    app: &Arc<AppContext>,
    db_table_wrapper: &Arc<DbTableWrapper>,
    partition_key: &String,
    row_keys: Vec<String>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;
    let write_access = db_table_wrapper.data.read().await;

    let db_partition = write_access.get_partition(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let mut db_rows = LazyVec::with_capacity(row_keys.len());

    for row_key in &row_keys {
        let db_row = db_partition.get_row(row_key);

        if let Some(db_row) = db_row {
            db_rows.add(db_row);
        }
    }

    return Ok(ReadOperationResult::compile_array_or_empty_from_partition(
        app,
        db_table_wrapper,
        partition_key,
        db_rows.get_result(),
        update_statistics,
    )
    .await);
}

/*
pub async fn get_single_partition_multiple_rows_with_no_expiration_time_update(
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let mut json_array_writer = JsonArrayWriter::new();

    for row_key in &row_keys {
        let db_row = db_partition.get_row(row_key, UpdatePartitionReadMoment::None);

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);
            json_array_writer.write_raw_element(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}

pub async fn get_single_partition_multiple_rows_with_expiration_time_update(
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
    update_expiration_time: UpdateExpirationTimeModel,
) -> ReadOperationResult {
}
 */
