use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

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

    let table_data = db_table.data.read().await;

    let get_partition_result = table_data.get_partition(partition_key);

    let result = match get_partition_result {
        Some(partition) => {
            let db_rows = super::super::read_filter::filter_it(
                partition.get_all_rows().into_iter(),
                limit,
                skip,
            );

            ReadOperationResult::compile_array_or_empty_from_partition(
                app,
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
