use std::collections::HashMap;

use my_azure_storage_sdk::AzureConnection;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{DbPartitionSnapshot, DbTable},
    persistence::blob_content_cache::BlobPartitionUpdateTimeResult,
};

pub async fn update_table(app: &AppContext, table_name: &str, azure_connection: &AzureConnection) {
    let db_table_result = app.db.get_table(table_name).await;
    let partitions_in_blob = app.blob_content_cache.get_snapshot(table_name).await;

    match db_table_result {
        Some(db_table) => {
            if partitions_in_blob.is_none() {
                // If We have partition in db but do not have in blob
                sync_table_to_blob(app, azure_connection, db_table.as_ref()).await;
            } else {
                //If We have some partitions in blob and some in Table - we synch the difference
                sync_partitions_difference(
                    app,
                    db_table.as_ref(),
                    azure_connection,
                    partitions_in_blob.unwrap(),
                )
                .await;
            }
        }
        None => {
            // If we do not have partition in DB but have in blob - we delete it from blob
            if partitions_in_blob.is_some() {
                super::blob::delete_table::with_retries(app, azure_connection, table_name).await;
                app.blob_content_cache.delete_table(table_name).await;
            }
        }
    }
}

async fn sync_partitions_difference(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureConnection,
    mut partitions_in_blob: HashMap<String, DateTimeAsMicroseconds>,
) {
    let partitions_in_db = db_table.get_partitions_update_time().await;

    for (partition_key, last_update_in_db) in &partitions_in_db {
        let last_update_in_blob = partitions_in_blob.get(partition_key);

        match last_update_in_blob {
            Some(last_update_in_blob) => {
                // If we have it in Db and in Blob - but it has different update Time = we sync it
                if last_update_in_blob.unix_microseconds != last_update_in_db.unix_microseconds {
                    let partition_snapshot = db_table.get_partition_snapshot(partition_key).await;

                    sync_partition_to_blob(
                        app,
                        azure_connection,
                        db_table.name.as_str(),
                        partition_key,
                        partition_snapshot,
                    )
                    .await
                }
            }
            None => {
                // If we do not have record in Blob - but have it in Db - we upload it
                let partition_snapshot = db_table.get_partition_snapshot(partition_key).await;

                if let Some(partition_snapshot) = partition_snapshot {
                    sync_partition_to_blob(
                        app,
                        azure_connection,
                        db_table.name.as_str(),
                        partition_key,
                        Some(partition_snapshot),
                    )
                    .await
                }
            }
        }
        partitions_in_blob.remove(partition_key);
    }

    for (partition_key, _) in &partitions_in_blob {
        //We have it in blob - but do not have it in DB. Remove it...
        sync_partition_to_blob(
            app,
            azure_connection,
            db_table.name.as_str(),
            partition_key,
            None,
        )
        .await;
    }
}

pub async fn update_partitions(
    app: &AppContext,
    table_name: &str,
    partitions: &[String],
    azure_connection: &AzureConnection,
) {
    let get_table_result = app.db.get_table(table_name).await;

    if get_table_result.is_none() {
        update_table(app, table_name, azure_connection).await;
        return;
    }

    let db_table = get_table_result.unwrap();

    for partition_key in partitions {
        sync_partition(app, azure_connection, db_table.as_ref(), partition_key).await;
    }
}

async fn sync_partition(
    app: &AppContext,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
    partition_key: &str,
) {
    let partition_in_blob = app
        .blob_content_cache
        .get(db_table.name.as_str(), partition_key)
        .await;

    match partition_in_blob {
        BlobPartitionUpdateTimeResult::Ok(update_time_in_blob) => {
            let update_time_in_db = db_table.get_partition_update_time(partition_key).await;

            if update_time_in_db.is_none() {
                sync_partition_to_blob(
                    app,
                    azure_connection,
                    db_table.name.as_str(),
                    partition_key,
                    None,
                )
                .await;
                return;
            }

            let update_time_in_db = update_time_in_db.unwrap();

            if update_time_in_db.unix_microseconds != update_time_in_blob.unix_microseconds {
                let partition_snapshot = db_table.get_partition_snapshot(partition_key).await;

                sync_partition_to_blob(
                    app,
                    azure_connection,
                    db_table.name.as_str(),
                    partition_key,
                    partition_snapshot,
                )
                .await;
            }
        }
        BlobPartitionUpdateTimeResult::TableNotFound => {
            update_table(app, db_table.name.as_str(), azure_connection).await;
            return;
        }
        BlobPartitionUpdateTimeResult::PartitionNoFound => {
            let partition_snapshot = db_table.get_partition_snapshot(partition_key).await;

            if let Some(partition_snapshot) = partition_snapshot {
                sync_partition_to_blob(
                    app,
                    azure_connection,
                    db_table.name.as_str(),
                    partition_key,
                    Some(partition_snapshot),
                )
                .await
            }
        }
    }
}

async fn sync_table_to_blob(
    app: &AppContext,
    azure_connection: &AzureConnection,
    db_table: &DbTable,
) {
    let table_snapshot = db_table.get_snapshot().await;
    super::blob::create_table::with_retries(
        app,
        azure_connection,
        db_table.name.as_str(),
        &table_snapshot.attr,
    )
    .await;

    app.blob_content_cache
        .create_table(db_table.name.as_str(), &table_snapshot)
        .await;

    for (partition_key, snapshot) in table_snapshot.partitions {
        sync_partition_to_blob(
            app,
            azure_connection,
            db_table.name.as_str(),
            partition_key.as_str(),
            Some(snapshot),
        )
        .await;
    }
}

async fn sync_partition_to_blob(
    app: &AppContext,
    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
    snapshot: Option<DbPartitionSnapshot>,
) {
    if snapshot.is_none() {
        delete_partition_from_blob(app, azure_connection, table_name, partition_key).await;
        return;
    }

    let snapshot = snapshot.unwrap();

    super::blob::save_partition::with_retries(
        app,
        azure_connection,
        table_name,
        partition_key,
        &snapshot.content,
    )
    .await;

    app.blob_content_cache
        .update_table_partition_snapshot_id(table_name, partition_key, snapshot.last_update)
        .await;
}

async fn delete_partition_from_blob(
    app: &AppContext,

    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
) {
    super::blob::delete_partition::with_retires(app, azure_connection, table_name, partition_key)
        .await;

    app.blob_content_cache
        .delete_table_partition(table_name, partition_key)
        .await;
}
