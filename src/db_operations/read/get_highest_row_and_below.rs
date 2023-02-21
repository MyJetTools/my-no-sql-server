use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

use super::ReadOperationResult;

pub async fn get_highest_row_and_below(
    app: &Arc<AppContext>,
    db_table_wrapper: &Arc<DbTableWrapper>,
    partition_key: &String,
    row_key: &String,
    limit: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::check_app_states(app)?;

    let read_access = db_table_wrapper.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below(row_key, limit);

    return Ok(ReadOperationResult::compile_array_or_empty_from_partition(
        db_table_wrapper,
        partition_key,
        db_rows,
        update_statistics,
    )
    .await);

    /*
    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in db_rows {
        json_array_writer.write_raw_element(db_row.data.as_ref());
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
     */
}

/*
async fn get_highest_row_and_below_with_no_expiration_time_update(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &String,
    limit: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below(row_key, Some(now), limit);

    if db_rows.len() == 0 {
        return ReadOperationResult::EmptyArray;
    }

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in db_rows {
        json_array_writer.write_raw_element(db_row.data.as_ref());
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}

async fn get_highest_row_and_below_and_update_expiration_time(
    db_table: &DbTableWrapper,
    partition_key: &str,
    row_key: &String,
    limit: Option<usize>,
    update_expiration_time: UpdateExpirationTimeModel,
) -> ReadOperationResult {
}
 */
