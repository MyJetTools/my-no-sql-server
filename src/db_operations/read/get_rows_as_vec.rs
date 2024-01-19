use std::sync::Arc;

use my_no_sql_sdk::core::{db::DbRow, db_json_entity::JsonTimeStamp};
use my_no_sql_server_core::DbTableWrapper;

use crate::{
    app::AppContext,
    db_operations::{DbOperationError, UpdateStatistics},
};

pub async fn execute(
    app: &Arc<AppContext>,
    table: &Arc<DbTableWrapper>,
    partition_key: Option<&String>,
    row_key: Option<&String>,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
    update_statistics: UpdateStatistics,
) -> Result<Vec<Arc<DbRow>>, DbOperationError> {
    super::super::check_app_states(app)?;

    if let Some(partition_key) = partition_key {
        if let Some(row_key) = row_key {
            match get_as_partition_key_and_row_key(
                table,
                partition_key,
                row_key,
                now,
                update_statistics,
            )
            .await
            {
                Some(result) => {
                    return Ok(vec![result]);
                }
                None => {
                    return Ok(vec![]);
                }
            }
        } else {
            return Ok(get_as_partition_key_only(table, partition_key, limit, skip, now).await);
        }
    }

    if let Some(row_key) = row_key {
        let result = get_as_row_key_only(table, row_key, limit, skip, now, update_statistics).await;
        return Ok(result);
    }

    return Ok(get_all(table, limit, skip).await);
}

pub async fn get_as_partition_key_and_row_key(
    table: &Arc<DbTableWrapper>,
    partition_key: &String,
    row_key: &String,
    now: &JsonTimeStamp,
    update_statistics: UpdateStatistics,
) -> Option<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key)?;

    let db_row = db_partition.get_row_and_clone(row_key)?;

    update_statistics.update(db_partition, Some(&db_row), now.date_time);

    Some(db_row)
}

async fn get_as_partition_key_only(
    table: &DbTableWrapper,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Vec<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key);

    if db_partition.is_none() {
        return vec![];
    }

    let db_partition = db_partition.unwrap();

    let result = super::read_filter::filter_it_and_clone(
        db_partition.get_all_rows().into_iter(),
        limit,
        skip,
        now.date_time,
    );

    result
}

async fn get_as_row_key_only(
    table: &Arc<DbTableWrapper>,
    row_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
    update_statistics: UpdateStatistics,
) -> Vec<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let mut result = Vec::new();

    for (db_partition, db_row) in read_access.get_by_row_key(row_key, skip, limit) {
        update_statistics.update(db_partition, Some(db_row), now.date_time);
        result.push(db_row.clone());
    }

    result
}

async fn get_all(
    table: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
) -> Vec<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let mut result = if let Some(limit) = limit {
        Vec::with_capacity(limit)
    } else {
        Vec::new()
    };

    for (_, db_row) in read_access.get_all_rows(skip, limit) {
        result.push(db_row.clone());
    }

    result
}
