use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

use super::super::ReadOperationResult;

pub async fn get_single(
    app: &Arc<AppContext>,
    db_table: &Arc<DbTableWrapper>,
    partition_key: &String,
    row_key: &String,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_data = db_table.data.read().await;

    let partition = table_data.get_partition(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(row_key);

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();

    if update_statistics.has_statistics_to_update() {
        update_statistics
            .update_statistics(app, db_table, partition_key, || [row_key].into_iter())
            .await;
    }

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}
