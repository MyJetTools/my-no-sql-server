use my_no_sql_core::db::{UpdateExpirationTimeModel, UpdatePartitionReadMoment};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db::DbTableWrapper, db_operations::DbOperationError};

use super::super::{read_filter, ReadOperationResult};

pub async fn get_all_by_row_key(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_all_by_row_key_and_update_expiration_time(
            db_table_wrapper,
            row_key,
            limit,
            skip,
            &update_expiration_time,
        )
        .await
    } else {
        get_all_by_row_key_and_update_no_expiration_time(db_table_wrapper, row_key, limit, skip)
            .await
    };

    Ok(result)
}

async fn get_all_by_row_key_and_update_no_expiration_time(
    db_table_wrapper: &DbTableWrapper,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table_wrapper.data.read().await;

    read_access.db_table.last_read_time.update(now);

    let mut result = Vec::new();

    let now = DateTimeAsMicroseconds::now();

    for partition in read_access.db_table.get_partitions() {
        let get_row_result = partition.get_row(
            row_key,
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
        );

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(read_filter::filter_it(
        result.into_iter(),
        limit,
        skip,
        now,
    ));
}

async fn get_all_by_row_key_and_update_expiration_time(
    db_table_wrapper: &DbTableWrapper,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut write_access = db_table_wrapper.data.write().await;

    write_access.db_table.last_read_time.update(now);

    let mut result = Vec::new();

    let now = DateTimeAsMicroseconds::now();

    for partition in write_access.db_table.partitions.values_mut() {
        let get_row_result = partition.get_row_and_update_expiration_time(
            row_key,
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
            update_expiration_time,
        );

        if let Some(db_row) = get_row_result {
            result.push(db_row);
        }
    }

    return ReadOperationResult::RowsArray(read_filter::filter_it(
        result.iter().map(|itm| itm),
        limit,
        skip,
        now,
    ));
}
