use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbTable, db_operations::DbOperationError};

use super::ReadOperationResult;

pub async fn get_all_table_rows(
    table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let result = table_data.iterate_all_rows();

    ReadOperationResult::RowsArray(super::read_filter::filter_it(result, limit, skip, now))
}

pub async fn get_all_rows_by_partition_key(
    table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let get_partition_result = table_data.get_partition(partition_key);

    match get_partition_result {
        Some(partition) => {
            partition.last_read_access.update(now);

            ReadOperationResult::RowsArray(super::read_filter::filter_it(
                partition.rows.values(),
                limit,
                skip,
                now,
            ))
        }
        None => ReadOperationResult::EmptyArray,
    }
}

pub async fn get_all_rows_by_row_key(
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let mut result = Vec::new();

    for partition in table_data.get_partitions() {
        let get_row_result = partition.rows.get(row_key);

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(super::read_filter::filter_it(
        result.into_iter(),
        limit,
        skip,
        now,
    ));
}

pub async fn get_row(
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

    let db_row = partition.get_row(row_key);

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();

    partition.last_read_access.update(now);
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}

pub async fn get_single_partition_multiple_rows(
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
        let db_row = db_partition.get_row(row_key);

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);

            json_array_writer.write_raw_element(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}
