use my_no_sql_sdk::core::db::db_table_master_node::PartitionLastWriteMoment;
use my_no_sql_server_core::{db_snapshots::DbTableSnapshot, DbTableWrapper};
use rust_extensions::sorted_vec::SortedVecWithStrKey;

use crate::app::AppContext;

use super::super::sync;

pub async fn save_table(app: &AppContext, db_table: &DbTableWrapper) {
    let snapshot: DbTableSnapshot = db_table.get_table_snapshot().await;

    let in_blob = app
        .blob_content_cache
        .get_snapshot(db_table.name.as_str())
        .await;

    match in_blob {
        Some(in_blob) => {
            sync_with_blob(app, db_table.name.as_str(), in_blob, snapshot).await;
        }
        None => {
            init_new_table(app, db_table.name.as_str(), snapshot).await;
        }
    }
}

async fn init_new_table(app: &AppContext, table_name: &str, snapshot: DbTableSnapshot) {
    for snapshot in snapshot.by_partition {
        sync::upload_partition(app, table_name, snapshot).await;
    }
}

async fn sync_with_blob(
    app: &AppContext,
    table_name: &str,
    mut in_blob: SortedVecWithStrKey<PartitionLastWriteMoment>,
    snapshot: DbTableSnapshot,
) {
    for partition_snapshot in snapshot.by_partition {
        match in_blob.remove(partition_snapshot.partition_key.as_str()) {
            Some(snapshot_in_blob) => {
                if partition_snapshot.has_to_persist(snapshot_in_blob.last_write_moment) {
                    sync::upload_partition(app, table_name, partition_snapshot).await;
                }
            }
            None => {
                sync::upload_partition(app, table_name, partition_snapshot).await;
            }
        }
    }

    for item in in_blob.into_vec() {
        sync::delete_partition(app, table_name, item.partition_key).await;
    }
}
