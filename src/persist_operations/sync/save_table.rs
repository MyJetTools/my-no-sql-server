use std::collections::HashMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::AppContext,
    db::{db_snapshots::DbTableSnapshot, DbTable},
};

use super::super::sync;

pub async fn save_table(app: &AppContext, db_table: &DbTable) {
    let snapshot = db_table.get_table_snapshot().await;

    let in_blob = app
        .blob_content_cache
        .get_snapshot(db_table.name.as_str())
        .await;

    match in_blob {
        Some(in_blob) => {
            sync_with_blob(app, db_table.name.as_str(), in_blob, &snapshot).await;
        }
        None => {
            init_new_table(app, db_table.name.as_str(), &snapshot).await;
        }
    }
}

async fn init_new_table(app: &AppContext, table_name: &str, snapshot: &DbTableSnapshot) {
    for (partition_key, db_partition_snapshot) in &snapshot.by_partition {
        sync::upload_partition(
            app,
            table_name,
            partition_key.as_str(),
            db_partition_snapshot,
        )
        .await;
    }
}

async fn sync_with_blob(
    app: &AppContext,
    table_name: &str,
    mut in_blob: HashMap<String, DateTimeAsMicroseconds>,
    snapshot: &DbTableSnapshot,
) {
    for (partition_key, partition_snapshot) in &snapshot.by_partition {
        match in_blob.remove(partition_key) {
            Some(snapshot_in_blob) => {
                if partition_snapshot.has_to_persist(snapshot_in_blob) {
                    sync::upload_partition(app, table_name, partition_key, partition_snapshot)
                        .await;
                }
            }
            None => {
                sync::upload_partition(app, table_name, partition_key, partition_snapshot).await;
            }
        }
    }

    for (partition_key, _) in in_blob {
        sync::delete_partition(app, table_name, partition_key.as_str()).await;
    }
}
