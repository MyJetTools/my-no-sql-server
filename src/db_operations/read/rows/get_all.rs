use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{db::DbTable, db_operations::read::read_filter::DbRowsFilter};

use super::super::ReadOperationResult;

pub async fn get_all(
    table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in DbRowsFilter::new(table_data.get_all_rows().into_iter(), limit, skip) {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    ReadOperationResult::RowsArray(json_array_writer.build())
}

pub async fn get_all_and_update_expiration_time(
    table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
    expiration_time: Option<DateTimeAsMicroseconds>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let result_items = table_data.get_all_rows_and_update_expiration_time(expiration_time);

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in DbRowsFilter::new(result_items.iter().map(|itm| itm), limit, skip) {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    ReadOperationResult::RowsArray(json_array_writer.build())
}
