use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::db::{DbTable, UpdatePartitionReadMoment};

use super::super::ReadOperationResult;

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
        let db_row = db_partition.get_row(row_key, UpdatePartitionReadMoment::None);

        if let Some(db_row) = db_row {
            db_row.update_last_access(now);
            json_array_writer.write_raw_element(&db_row.data);
        }
    }

    return ReadOperationResult::RowsArray(json_array_writer.build());
}
