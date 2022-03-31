use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::DbTable;

use super::super::{read_filter, ReadOperationResult};

pub async fn get_all_by_partition_key(
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
        Some(partition) => ReadOperationResult::RowsArray(read_filter::filter_it(
            partition.get_all_rows(Some(now)),
            limit,
            skip,
            now,
        )),
        None => ReadOperationResult::EmptyArray,
    }
}

pub async fn get_all_by_partition_key_and_update_expiration_time(
    table: &DbTable,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    expiration_time: Option<DateTimeAsMicroseconds>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let get_partition_result = table_data.get_partition_mut(partition_key);

    match get_partition_result {
        Some(partition) => ReadOperationResult::RowsArray(read_filter::filter_it(
            partition
                .get_all_rows_and_update_expiration_time(Some(now), expiration_time)
                .iter()
                .map(|itm| itm),
            limit,
            skip,
            now,
        )),
        None => ReadOperationResult::EmptyArray,
    }
}
