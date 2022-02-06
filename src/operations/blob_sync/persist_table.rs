use my_azure_storage_sdk::AzureStorageConnection;

use crate::{app::AppContext, db::DbTable};

pub async fn execute(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureStorageConnection,
) {
    let table_in_blob = app
        .blob_content_cache
        .get_snapshot(db_table.name.as_str())
        .await;

    if table_in_blob.is_none() {
        from_no_table_in_blob(app, db_table, azure_connection).await;
        return;
    }

    let table_in_blob = table_in_blob.unwrap();

    let mut table_snapshot = db_table.get_table_snapshot().await;

    for (partition_key, last_write_time) in table_in_blob {
        let partition_snapshot = table_snapshot.by_partition.remove(partition_key.as_str());

        super::persist_partition::sync_single_partition(
            app,
            db_table.name.as_str(),
            partition_key.as_str(),
            azure_connection,
            partition_snapshot.as_ref(),
            Some(last_write_time),
        )
        .await;
    }

    for (partition_key, db_partition) in &table_snapshot.by_partition {
        super::persist_partition::sync_single_partition(
            app,
            db_table.name.as_str(),
            partition_key.as_str(),
            azure_connection,
            Some(db_partition),
            None,
        )
        .await;
    }
}

pub async fn from_no_table_in_blob(
    app: &AppContext,
    db_table: &DbTable,
    azure_connection: &AzureStorageConnection,
) {
    let attr = db_table.attributes.get_snapshot();
    crate::blob_operations::create_table::with_retries(
        app,
        azure_connection,
        db_table.name.as_str(),
        &attr,
    )
    .await;

    let table_snapshot = db_table.get_table_snapshot().await;

    for partition_key in table_snapshot.by_partition.keys() {
        if let Some(db_partition_snapshot) = &db_table.get_partition_snapshot(partition_key).await {
            crate::blob_operations::save_partition::with_retries(
                app,
                azure_connection,
                db_table.name.as_str(),
                partition_key.as_str(),
                db_partition_snapshot,
            )
            .await;
        } else {
            crate::blob_operations::delete_partition::with_retires(
                app,
                azure_connection,
                db_table.name.as_str(),
                partition_key,
            )
            .await
        }
    }
}
