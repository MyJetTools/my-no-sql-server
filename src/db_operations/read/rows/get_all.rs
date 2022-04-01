use my_json::json_writer::JsonArrayWriter;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    db::{DbTable, UpdateExpirationTimeModel},
    db_operations::read::read_filter::DbRowsFilter,
};

use super::super::ReadOperationResult;

pub async fn get_all(
    table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> ReadOperationResult {
    if let Some(update_expiration_time) = update_expiration_time {
        get_all_and_update_expiration_time(table, limit, skip, &update_expiration_time).await
    } else {
        get_all_and_no_expiration_time_update(table, limit, skip).await
    }
}

async fn get_all_and_no_expiration_time_update(
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

async fn get_all_and_update_expiration_time(
    table: &DbTable,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut table_data = table.data.write().await;

    table_data.last_read_time.update(now);

    let result_items = table_data.get_all_rows_and_update_expiration_time(update_expiration_time);

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in DbRowsFilter::new(result_items.iter().map(|itm| itm), limit, skip) {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    ReadOperationResult::RowsArray(json_array_writer.build())
}
