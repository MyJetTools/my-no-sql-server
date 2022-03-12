use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{db_snapshots::DbPartitionSnapshot, DbTable},
    persist_operations::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn execute(app: &AppContext, db_table: &DbTable, partition_key: &str) {
    let get_blob_content_cache = app
        .blob_content_cache
        .get(db_table.name.as_str(), partition_key)
        .await;

    let partition_snapshot = db_table.get_partition_snapshot(partition_key).await;

    match get_blob_content_cache {
        BlobPartitionUpdateTimeResult::Ok(blob_date_time) => {
            sync_single_partition(
                app,
                db_table.name.as_str(),
                partition_key,
                partition_snapshot.as_ref(),
                Some(blob_date_time),
            )
            .await;
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {
            super::persist_table::from_no_table_in_blob(app, db_table).await;
        }
        BlobPartitionUpdateTimeResult::PartitionNoFound => {
            if let Some(snapshot) = partition_snapshot {
                app.persist_io
                    .save_partition(db_table.name.as_str(), partition_key, &snapshot)
                    .await;
            }
        }
    }
}

pub async fn sync_single_partition(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    partition_snapshot: Option<&DbPartitionSnapshot>,
    blob_date_time: Option<DateTimeAsMicroseconds>,
) {
    if let Some(db_partition_snapshot) = partition_snapshot {
        if let Some(blob_date_time) = blob_date_time {
            if db_partition_snapshot.last_write_moment.unix_microseconds
                > blob_date_time.unix_microseconds
            {
                app.persist_io
                    .save_partition(table_name, partition_key, db_partition_snapshot)
                    .await;
            }
        } else {
            app.persist_io
                .save_partition(table_name, partition_key, db_partition_snapshot)
                .await;
        }
    } else {
        app.persist_io
            .delete_partition(table_name, partition_key)
            .await;
    }
}
