use std::sync::Arc;

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::date_time::DateTimeAsMicroseconds;

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

    let now = DateTimeAsMicroseconds::now();

    let table_data = db_table.data.read().await;

    table_data.last_read_time.update(now);

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

/*
async fn get_single_and_update_no_expiration_time(
    table: &DbTableWrapper,
    partition_key: &str,
    row_key: &str,
) -> Result<ReadOperationResult, DbOperationError> {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let partition = table_data.get_partition(partition_key);

    if partition.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let partition = partition.unwrap();

    let db_row = partition.get_row(
        row_key,
        UpdatePartitionReadMoment::UpdateIfElementIsFound(now),
    );

    if db_row.is_none() {
        return Err(DbOperationError::RecordNotFound);
    }

    let db_row = db_row.unwrap();
    db_row.last_read_access.update(now);

    return Ok(ReadOperationResult::SingleRow(db_row.data.clone()));
}

async fn get_single_and_update_expiration_time(
    table: &DbTableWrapper,
    partition_key: &str,
    row_key: &str,
    update_expiration_time: &UpdateExpirationTimeModel,
) -> Result<ReadOperationResult, DbOperationError> {
}
 */
