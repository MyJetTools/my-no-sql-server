use std::sync::Arc;

use my_no_sql_core::{
    db::{DbRow, UpdatePartitionReadMoment},
    db_json_entity::JsonTimeStamp,
};

use crate::{app::AppContext, db::DbTableWrapper, db_operations::DbOperationError};

use super::read_filter::DbRowsFilter;

pub async fn execute(
    app: &AppContext,
    db_table_wrapper: &DbTableWrapper,
    partition_key: Option<&String>,
    row_key: Option<&String>,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Result<Option<Vec<Arc<DbRow>>>, DbOperationError> {
    super::super::check_app_states(app)?;

    if let Some(partition_key) = partition_key {
        if let Some(row_key) = row_key {
            match get_as_partition_key_and_row_key(db_table_wrapper, partition_key, row_key, now)
                .await
            {
                Some(result) => {
                    return Ok(Some(vec![result]));
                }
                None => {
                    return Ok(None);
                }
            }
        } else {
            return Ok(get_as_partition_key_only(
                db_table_wrapper,
                partition_key,
                limit,
                skip,
                now,
            )
            .await);
        }
    }

    if let Some(row_key) = row_key {
        return Ok(get_as_row_key_only(db_table_wrapper, row_key, limit, skip, now).await);
    }

    return Ok(get_all(db_table_wrapper, limit, skip, now).await);
}

pub async fn get_as_partition_key_and_row_key(
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    row_key: &String,
    now: &JsonTimeStamp,
) -> Option<Arc<DbRow>> {
    let read_access = db_table_wrapper.data.read().await;

    let db_partition = read_access.db_table.get_partition(partition_key)?;

    let db_row = db_partition.get_row_and_clone(row_key, Some(now.date_time))?;

    db_row.last_read_access.update(now.date_time);

    Some(db_row)
}

async fn get_as_partition_key_only(
    db_table_wrapper: &DbTableWrapper,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = db_table_wrapper.data.read().await;

    let db_partition = read_access.db_table.get_partition(partition_key)?;

    let db_row_filter =
        DbRowsFilter::new(db_partition.get_all_rows(Some(now.date_time)), limit, skip);

    let mut result = None;

    for db_row in db_row_filter {
        if result.is_none() {
            result = Some(Vec::new());
        }

        result.as_mut().unwrap().push(db_row.clone());
        db_row.last_read_access.update(now.date_time);
    }

    result
}

async fn get_as_row_key_only(
    db_table_wrapper: &DbTableWrapper,
    row_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = db_table_wrapper.data.read().await;

    read_access.db_table.last_read_time.update(now.date_time);

    let mut data_by_row = Vec::new();

    for partition in read_access.db_table.get_partitions() {
        let get_row_result = partition.get_row(
            row_key,
            UpdatePartitionReadMoment::UpdateIfElementIsFound(now.date_time),
        );

        if let Some(db_row) = get_row_result {
            data_by_row.push(db_row);
        }
    }

    let mut result = None;

    let db_row_filter = DbRowsFilter::new(data_by_row.into_iter(), limit, skip);

    for db_row in db_row_filter {
        if result.is_none() {
            result = Some(Vec::new());
        }

        result.as_mut().unwrap().push(db_row.clone());
        db_row.last_read_access.update(now.date_time);
    }

    result
}

async fn get_all(
    db_table_wrapper: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = db_table_wrapper.data.read().await;

    let db_row_filter =
        DbRowsFilter::new(read_access.db_table.get_all_rows().into_iter(), limit, skip);

    let mut result = None;

    for db_row in db_row_filter {
        if result.is_none() {
            result = Some(Vec::new());
        }

        result.as_mut().unwrap().push(db_row.clone());
        db_row.last_read_access.update(now.date_time);
    }

    result
}
