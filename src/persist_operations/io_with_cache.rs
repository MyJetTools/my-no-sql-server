use std::sync::Arc;

use crate::db::db_snapshots::DbPartitionSnapshot;
use crate::db::DbTable;
use crate::AppContext;
use rust_extensions::StopWatch;

pub async fn save_partition(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    db_partition_snapshot: &DbPartitionSnapshot,
) {
    let mut stop_watch = StopWatch::new();
    stop_watch.start();

    let partition_file_name = super::serializers::blob_file_name::encode(partition_key);

    app.persist_io
        .save_table_file(table_name, partition_key, db_partition_snapshot)
        .await;

    stop_watch.pause();

    app.logs.add_info(
        Some(table_name.to_string()),
        crate::app::logs::SystemProcess::PersistOperation,
        "save_partition".to_string(),
        format!(
            "Saved partition {}/{} in {}",
            table_name,
            partition_key,
            stop_watch.duration_as_string()
        ),
    );

    app.blob_content_cache
        .update_table_partition_snapshot_id(
            table_name,
            partition_key,
            db_partition_snapshot.last_write_moment,
        )
        .await;
}

pub async fn delete_partition(app: &AppContext, table_name: &str, partition_key: &str) {
    app.persist_io
        .delete_partition(table_name, partition_key)
        .await;

    app.blob_content_cache
        .delete_table_partition(table_name, partition_key)
        .await;
}

pub async fn save_table_attributes(app: &AppContext, db_table: &DbTable) {
    if !app
        .blob_content_cache
        .has_table(db_table.name.as_str())
        .await
    {
        let attr = db_table.attributes.get_snapshot();
        app.persist_io
            .create_table(db_table.name.as_str(), &attr)
            .await;

        app.blob_content_cache
            .create_table(db_table.name.as_str(), attr)
            .await;
    } else {
        let attr = db_table.attributes.get_snapshot();
        app.persist_io
            .save_table_attributes(db_table.name.as_str(), &attr)
            .await;

        app.blob_content_cache
            .update_table_attributes(db_table.name.as_str(), attr)
            .await;
    }
}

pub async fn delete_table(app: Arc<AppContext>, table_name: String) {
    app.persist_io.delete_table(table_name.as_str()).await;

    app.blob_content_cache
        .delete_table(table_name.as_str())
        .await;

    app.logs.add_info(
        Some(table_name.to_string()),
        crate::app::logs::SystemProcess::PersistOperation,
        "delete_table".to_string(),
        "Saved".to_string(),
    );
}
