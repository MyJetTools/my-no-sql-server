use crate::{app::AppContext, db::DbTable};

pub async fn execute(app: &AppContext, db_table: &DbTable) {
    let table_in_blob = app
        .blob_content_cache
        .get_snapshot(db_table.name.as_str())
        .await;

    if table_in_blob.is_none() {
        from_no_table_in_blob(app, db_table).await;
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
            Some(db_partition),
            None,
        )
        .await;
    }
}

pub async fn from_no_table_in_blob(app: &AppContext, db_table: &DbTable) {
    let attr = db_table.attributes.get_snapshot();

    app.persist_io
        .create_table(db_table.name.as_str(), &attr)
        .await;

    app.blob_content_cache
        .create_table(db_table.name.as_str(), attr.clone())
        .await;

    app.logs.add_info(
        Some(db_table.name.to_string()),
        crate::app::logs::SystemProcess::PersistOperation,
        "create_table".to_string(),
        "Saved".to_string(),
    );

    let table_snapshot = db_table.get_table_snapshot().await;

    for partition_key in table_snapshot.by_partition.keys() {
        if let Some(db_partition_snapshot) = &db_table.get_partition_snapshot(partition_key).await {
            super::io_with_cache::save_partition(
                app,
                db_table.name.as_str(),
                partition_key.as_str(),
                db_partition_snapshot,
            )
            .await;
        } else {
            app.persist_io
                .delete_partition(db_table.name.as_str(), partition_key)
                .await;
        }
    }
}
