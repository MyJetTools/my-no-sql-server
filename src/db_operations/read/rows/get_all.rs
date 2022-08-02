use my_json::json_writer::JsonArrayWriter;
use my_no_sql_core::db::UpdateExpirationTimeModel;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::DbTableWrapper,
    db_operations::{read::read_filter::DbRowsFilter, DbOperationError},
};

use super::super::ReadOperationResult;

pub async fn get_all(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_all_and_update_expiration_time(db_table_wrapper, limit, skip, &update_expiration_time)
            .await
    } else {
        get_all_and_no_expiration_time_update(db_table_wrapper, limit, skip).await
    };

    Ok(result)
}

async fn get_all_and_no_expiration_time_update(
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table_wrapper.data.read().await;

    read_access.db_table.last_read_time.update(now);

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in DbRowsFilter::new(read_access.db_table.get_all_rows().into_iter(), limit, skip) {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    ReadOperationResult::RowsArray(json_array_writer.build())
}

async fn get_all_and_update_expiration_time(
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut write_access = db_table_wrapper.data.write().await;

    write_access.db_table.last_read_time.update(now);

    let result_items = write_access
        .db_table
        .get_all_rows_and_update_expiration_time(update_expiration_time);

    let mut json_array_writer = JsonArrayWriter::new();

    for db_row in DbRowsFilter::new(result_items.iter().map(|itm| itm), limit, skip) {
        json_array_writer.write_raw_element(&db_row.data);
        db_row.last_read_access.update(now);
    }

    ReadOperationResult::RowsArray(json_array_writer.build())
}
