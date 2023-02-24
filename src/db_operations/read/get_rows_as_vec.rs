use std::sync::Arc;

use my_no_sql_core::{db::DbRow, db_json_entity::JsonTimeStamp};
use my_no_sql_server_core::DbTableWrapper;

use crate::{app::AppContext, db_operations::DbOperationError};

pub async fn execute(
    app: &AppContext,
    table: &DbTableWrapper,
    partition_key: Option<&String>,
    row_key: Option<&String>,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Result<Option<Vec<Arc<DbRow>>>, DbOperationError> {
    super::super::check_app_states(app)?;

    if let Some(partition_key) = partition_key {
        if let Some(row_key) = row_key {
            match get_as_partition_key_and_row_key(table, partition_key, row_key, now).await {
                Some(result) => {
                    return Ok(Some(vec![result]));
                }
                None => {
                    return Ok(None);
                }
            }
        } else {
            return Ok(get_as_partition_key_only(table, partition_key, limit, skip, now).await);
        }
    }

    if let Some(row_key) = row_key {
        return Ok(get_as_row_key_only(table, row_key, limit, skip, now).await);
    }

    return Ok(get_all(table, limit, skip, now).await);
}

pub async fn get_as_partition_key_and_row_key(
    table: &DbTableWrapper,
    partition_key: &String,
    row_key: &String,
    now: &JsonTimeStamp,
) -> Option<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key)?;

    let db_row = db_partition.get_row_and_clone(row_key)?;

    db_row.last_read_access.update(now.date_time);

    Some(db_row)
}

async fn get_as_partition_key_only(
    table: &DbTableWrapper,
    partition_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key)?;

    super::read_filter::filter_it_and_clone(
        db_partition.get_all_rows().into_iter(),
        limit,
        skip,
        now.date_time,
    )
}

async fn get_as_row_key_only(
    table: &DbTableWrapper,
    row_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = table.data.read().await;

    read_access.last_read_time.update(now.date_time);

    let mut data_by_row = Vec::new();

    for partition in read_access.get_partitions() {
        let get_row_result = partition.get_row(row_key);

        if let Some(db_row) = get_row_result {
            data_by_row.push(db_row);
        }
    }

    super::read_filter::filter_it_and_clone(data_by_row.into_iter(), limit, skip, now.date_time)

    /*
    let db_row_filter = DbRowsFilter::new(data_by_row.into_iter(), limit, skip);

    for db_row in db_row_filter {
        if result.is_none() {
            result = Some(Vec::new());
        }

        result.as_mut().unwrap().push(db_row.clone());

    }

    result
     */
}

async fn get_all(
    table: &DbTableWrapper,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = table.data.read().await;

    super::read_filter::filter_it_and_clone(
        read_access.get_all_rows().into_iter(),
        limit,
        skip,
        now.date_time,
    )
}
