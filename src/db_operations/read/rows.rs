use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbTable, json::JsonArrayBuilder};

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

    ReadOperationResult::RowsArray(super::read_filter::filter_it(result, limit, skip))
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

    let get_partition_result = table_data.partitions.get(partition_key);

    match get_partition_result {
        Some(partition) => {
            partition.last_read_access.update(now);

            ReadOperationResult::RowsArray(super::read_filter::filter_it(
                partition.rows.values(),
                limit,
                skip,
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

    for partition in table_data.partitions.values() {
        let get_row_result = partition.rows.get(row_key);

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(super::read_filter::filter_it(
        result.into_iter(),
        limit,
        skip,
    ));
}

pub async fn get_row(table: &DbTable, partition_key: &str, row_key: &str) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let partition = table_data.partitions.get(partition_key);

    if partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(row_key);

    if db_row.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_row = db_row.unwrap();

    partition.last_read_access.update(now);
    db_row.last_read_access.update(now);

    return ReadOperationResult::SingleRow(db_row.data.clone());
}

pub async fn get_single_partition_multiple_rows(
    table: &DbTable,
    partition_key: &str,
    row_keys: Vec<String>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();
    let read_access = table.data.read().await;

    let db_partition = read_access.partitions.get(partition_key);

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let mut result = JsonArrayBuilder::new();

    for row_key in &row_keys {
        let db_row = db_partition.get_row(row_key);

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);

            result.append_json_object(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(result.build());
}
