use std::{collections::HashMap, sync::Arc};

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
) -> Result<Option<Vec<Arc<DbRow>>>, DbOperationError> {
    super::super::check_app_states(app)?;

    if let Some(partition_key) = partition_key {
        if let Some(row_key) = row_key {
            match get_as_partition_key_and_row_key(
                app,
                table,
                partition_key,
                row_key,
                now,
                update_statistics,
            )
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
            return Ok(get_as_partition_key_only(table, partition_key, limit, skip, now).await);
        }
    }

    if let Some(row_key) = row_key {
        return Ok(
            get_as_row_key_only(app, table, row_key, limit, skip, now, update_statistics).await,
        );
    }

    return Ok(get_all(table, limit, skip, now).await);
}

pub async fn get_as_partition_key_and_row_key(
    app: &Arc<AppContext>,
    table: &Arc<DbTableWrapper>,
    partition_key: &String,
    row_key: &String,
    now: &JsonTimeStamp,
    update_statistics: UpdateStatistics,
) -> Option<Arc<DbRow>> {
    let read_access = table.data.read().await;

    let db_partition = read_access.get_partition(partition_key)?;

    let db_row = db_partition.get_row_and_clone(row_key)?;

    if update_statistics.has_statistics_to_update() {
        update_statistics
            .update_statistics(app, table, partition_key, || [row_key.as_str()].into_iter())
            .await;
    }

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
    app: &Arc<AppContext>,
    table: &Arc<DbTableWrapper>,
    row_key: &String,
    limit: Option<usize>,
    skip: Option<usize>,
    now: &JsonTimeStamp,
    update_statistics: UpdateStatistics,
) -> Option<Vec<Arc<DbRow>>> {
    let read_access = table.data.read().await;

    let mut data_by_row = Vec::new();

    for partition in read_access.get_partitions() {
        let get_row_result = partition.get_row(row_key);

        if let Some(db_row) = get_row_result {
            data_by_row.push(db_row);
        }
    }

    let result = super::read_filter::filter_it_and_clone(
        data_by_row.into_iter(),
        limit,
        skip,
        now.date_time,
    );

    if let Some(result) = &result {
        if update_statistics.has_statistics_to_update() {
            let mut by_partition = HashMap::new();

            for db_row in result {
                let partition_key = db_row.get_partition_key();

                if !by_partition.contains_key(partition_key) {
                    by_partition.insert(partition_key.to_string(), Vec::new());
                }

                by_partition.get_mut(partition_key).unwrap().push(db_row);
            }

            for (partition_key, row_keys) in by_partition {
                update_statistics
                    .update_statistics(app, table, &partition_key, || {
                        row_keys.iter().map(|itm| itm.get_row_key())
                    })
                    .await;
            }
        }
    }

    result
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
