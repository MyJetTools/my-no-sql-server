use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::{read_as_json::DbEntityAsJsonArray, DbRow, DbTable};

use super::ReadOperationResult;

//TODO - Unit test it
pub async fn execute(
    db_table: &DbTable,
    partition_key: &str,
    row_key: &String,
    max_amount: usize,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table.data.read().await;

    let db_partition = read_access.get_partition(partition_key, Some(now));

    if db_partition.is_none() {
        return ReadOperationResult::EmptyArray;
    }

    let db_partition = db_partition.unwrap();

    let db_rows = db_partition.get_highest_row_and_below(row_key);

    if db_rows.len() == 0 {
        return ReadOperationResult::EmptyArray;
    }

    let result = reverse_and_take(db_rows, max_amount);

    return ReadOperationResult::RowsArray(result.as_json_array(Some(now)));
}

fn reverse_and_take(mut src: Vec<Arc<DbRow>>, max_amount: usize) -> Vec<Arc<DbRow>> {
    let mut result = Vec::new();

    for index in src.len() - 1..0 {
        let db_row = src.remove(index);
        result.push(db_row);

        if result.len() >= max_amount {
            break;
        }
    }

    result
}
