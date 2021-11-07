use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbPartition, DbRow, DbTableData},
    db_sync::states::DeleteRowsEventSyncData,
};

pub struct RemoveRowResult {
    pub removed_row: Arc<DbRow>,
    pub partition_is_empty: bool,
}

#[inline]
pub async fn remove_db_row(
    app: &AppContext,
    table_name: &str,
    db_partition: &mut DbPartition,
    row_key: &str,
    now: DateTimeAsMicroseconds,
) -> Option<RemoveRowResult> {
    let removed_row = db_partition.remove_row(row_key, now);

    if let Some(removed_row) = removed_row {
        app.rows_with_expiration
            .removed(table_name, removed_row.as_ref())
            .await;

        return Some(RemoveRowResult {
            partition_is_empty: db_partition.is_empty(),
            removed_row,
        });
    }

    None
}

pub fn handle_after_delete_row(
    table_data: &mut DbTableData,
    partition_key: &str,
    remove_row_result: &RemoveRowResult,
    sync_data: Option<&mut DeleteRowsEventSyncData>,
) {
    if !remove_row_result.partition_is_empty {
        return;
    }

    let removed_partition = table_data.partitions.remove(partition_key);

    if removed_partition.is_none() {
        return;
    }

    let removed_partition = removed_partition.unwrap();

    if let Some(sync_data) = sync_data {
        sync_data.new_deleted_partition(partition_key.to_string(), removed_partition);
    }
}

#[inline]
pub async fn bulk_remove_db_rows<'s, TIter: Iterator<Item = &'s String>>(
    app: &AppContext,
    table_name: &str,
    db_partition: &mut DbPartition,
    row_keys: TIter,
    now: DateTimeAsMicroseconds,
) -> Option<Vec<Arc<DbRow>>> {
    let mut removed_rows = Vec::new();
    for row_key in row_keys {
        let removed_row = db_partition.remove_row(row_key, now);

        if let Some(removed_row) = removed_row {
            removed_rows.push(removed_row);
        }
    }

    if removed_rows.len() > 0 {
        app.rows_with_expiration
            .bulk_removed(table_name, removed_rows.iter())
            .await;

        return Some(removed_rows);
    }

    None
}

#[inline]
pub async fn insert_db_row(
    app: &AppContext,
    table_name: &str,
    db_partition: &mut DbPartition,
    db_row: Arc<DbRow>,
) {
    let write_access_moment = db_row.time_stamp;
    db_partition.insert(db_row.clone(), Some(write_access_moment));

    app.rows_with_expiration.add(table_name, db_row).await;
}

#[inline]
pub async fn bulk_insert_db_rows(
    app: &AppContext,
    table_name: &str,
    db_partition: &mut DbPartition,
    db_rows: &[Arc<DbRow>],
    write_access_moment: DateTimeAsMicroseconds,
) {
    db_partition.bulk_insert(db_rows, Some(write_access_moment));

    app.rows_with_expiration
        .add_multiple(table_name, db_rows)
        .await;
}

#[inline]
pub async fn remove_partition(
    app: &AppContext,
    db_table_data: &mut DbTableData,
    partition_key: &str,
) -> Option<DbPartition> {
    let db_partition = db_table_data.partitions.remove(partition_key)?;

    app.rows_with_expiration
        .bulk_removed(db_table_data.name.as_str(), db_partition.rows.values())
        .await;

    Some(db_partition)
}

#[inline]
pub async fn clean_table(
    app: &AppContext,
    db_table_data: &mut DbTableData,
) -> Option<Vec<DbPartition>> {
    let partitions: Vec<String> = db_table_data
        .partitions
        .keys()
        .map(|itm| itm.to_string())
        .collect();

    let mut result = Vec::new();

    for partition_key in &partitions {
        let removed_partition = remove_partition(app, db_table_data, &partition_key).await;
        if let Some(removed_partition) = removed_partition {
            result.push(removed_partition)
        }
    }

    if result.len() == 0 {
        return None;
    }

    return Some(result);
}
