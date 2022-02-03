use my_azure_storage_sdk::AzureStorageConnection;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbPartitionSnapshot, DbTable},
    persistence::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureStorageConnection,
    partition_key: &str,
) {
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
                azure_connection,
                partition_snapshot,
                Some(blob_date_time),
            )
            .await;
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {
            super::sync_table::from_no_table_in_blob(app, db_table, azure_connection).await;
        }
        BlobPartitionUpdateTimeResult::PartitionNoFound => {
            if let Some(snapshot) = partition_snapshot {
                crate::blob_operations::save_partition::with_retries(
                    app,
                    azure_connection,
                    db_table.name.as_str(),
                    partition_key,
                    snapshot,
                )
                .await;
            }
        }
    }
}

pub async fn sync_single_partition(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    azure_connection: &AzureStorageConnection,
    partition_snapshot: Option<DbPartitionSnapshot>,
    blob_date_time: Option<DateTimeAsMicroseconds>,
) {
    if let Some(db_partition_snapshot) = partition_snapshot {
        if let Some(blob_date_time) = blob_date_time {
            if db_partition_snapshot.last_write_moment.unix_microseconds
                > blob_date_time.unix_microseconds
            {
                crate::blob_operations::save_partition::with_retries(
                    app,
                    azure_connection,
                    table_name,
                    partition_key,
                    db_partition_snapshot,
                )
                .await;
            }
        } else {
            crate::blob_operations::save_partition::with_retries(
                app,
                azure_connection,
                table_name,
                partition_key,
                db_partition_snapshot,
            )
            .await;
        }
    } else {
        crate::blob_operations::delete_partition::with_retires(
            app,
            azure_connection,
            table_name,
            partition_key,
        )
        .await
    }
}
