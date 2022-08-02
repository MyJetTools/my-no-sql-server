use my_no_sql_core::db::UpdateExpirationTimeModel;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, db::DbTableWrapper, db_operations::DbOperationError};

use super::super::{read_filter, ReadOperationResult};

pub async fn get_all_by_partition_key(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: Option<UpdateExpirationTimeModel>,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let result = if let Some(update_expiration_time) = update_expiration_time {
        get_all_by_partition_key_and_update_expiration_time(
            db_table_wrapper,
            partition_key,
            limit,
            skip,
            &update_expiration_time,
        )
        .await
    } else {
        get_all_by_partition_key_and_no_updates(db_table_wrapper, partition_key, limit, skip).await
    };

    Ok(result)
}

pub async fn get_all_by_partition_key_and_no_updates(
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let read_access = db_table_wrapper.data.read().await;

    read_access.db_table.last_read_time.update(now);

    let get_partition_result = read_access.db_table.get_partition(partition_key);

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
    db_table_wrapper: &DbTableWrapper,
    partition_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let mut write_access = db_table_wrapper.data.write().await;

    write_access.db_table.last_read_time.update(now);

    let get_partition_result = write_access.db_table.get_partition_mut(partition_key);

    match get_partition_result {
        Some(partition) => ReadOperationResult::RowsArray(read_filter::filter_it(
            partition
                .get_all_rows_and_update_expiration_time(Some(now), update_expiration_time)
                .iter()
                .map(|itm| itm),
            limit,
            skip,
            now,
        )),
        None => ReadOperationResult::EmptyArray,
    }
}
