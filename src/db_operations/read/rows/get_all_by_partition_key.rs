use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

use super::super::ReadOperationResult;

pub async fn get_all_by_partition_key(
    app: &Arc<AppContext>,
    db_table: &Arc<DbTableWrapper>,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let now = DateTimeAsMicroseconds::now();

    let mut table_data = db_table.data.write().await;

    table_data.last_read_time.update(now);

    let get_partition_result = table_data.get_partition_mut(partition_key);

    let result = match get_partition_result {
        Some(partition) => {
            let db_rows = super::super::read_filter::filter_it(
                partition.get_all_rows().into_iter(),
                limit,
                skip,
            );

            ReadOperationResult::compile_array_or_empty_from_partition(
                db_table,
                partition_key,
                db_rows,
                update_statistics,
            )
            .await
        }
        None => ReadOperationResult::EmptyArray,
    };

    Ok(result)
}

/*
pub async fn get_all_by_partition_key_and_no_updates(
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
    update_expiration_time: &UpdateExpirationTimeModel,
) -> ReadOperationResult {
}
 */
