use std::{collections::HashMap, sync::Arc};

use my_no_sql_server_core::DbTableWrapper;
use rust_extensions::lazy::LazyVec;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

use super::super::ReadOperationResult;

pub async fn get_all_by_row_key(
    app: &Arc<AppContext>,
    db_table: &Arc<DbTableWrapper>,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> Result<ReadOperationResult, DbOperationError> {
    super::super::super::check_app_states(app)?;

    let table_data = db_table.data.read().await;

    let mut db_rows = LazyVec::new();

    for partition in table_data.partitions.get_partitions() {
        let get_row_result = partition.get_row(row_key);

        if let Some(db_row) = get_row_result {
            db_rows.add(db_row);
        }
    }

    let db_rows = db_rows.get_result();

    if db_rows.is_none() {
        return Ok(ReadOperationResult::EmptyArray);
    }

    let db_rows = super::super::read_filter::filter_it(db_rows.unwrap().into_iter(), limit, skip);

    let db_rows = if let Some(db_rows) = db_rows {
        let mut result = HashMap::new();
        for db_row in db_rows {
            result.insert(db_row.partition_key.to_string(), vec![db_row]);
        }

        Some(result)
    } else {
        None
    };

    return Ok(ReadOperationResult::compile_array_or_empty(
        app,
        db_table,
        db_rows,
        update_statistics,
    )
    .await);
}

/*
async fn get_all_by_row_key_and_update_no_expiration_time(
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
) -> ReadOperationResult {
    let now = DateTimeAsMicroseconds::now();

    let table_data = table.data.read().await;

    table_data.last_read_time.update(now);

    let mut result = Vec::new();

    let now = DateTimeAsMicroseconds::now();

    for partition in table_data.get_partitions() {
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
    table: &DbTable,
    row_key: &str,
    limit: Option<usize>,
    skip: Option<usize>,
    update_statistics: UpdateStatistics,
) -> ReadOperationResult {
}
 */
