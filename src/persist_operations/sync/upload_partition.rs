use crate::{app::AppContext, db::db_snapshots::DbPartitionSnapshot, persist_io::TableFile};

pub async fn upload_partition(
    app: &AppContext,
    table_name: &str,
    partition_key: &str,
    snapshot: &DbPartitionSnapshot,
) {
    let content = snapshot.db_rows_snapshot.as_json_array();

    app.persist_io
        .save_table_file(
            table_name,
            &TableFile::DbPartition(partition_key.to_string()),
            content.build(),
        )
        .await;

    app.blob_content_cache
        .update_table_partition_snapshot_id(table_name, partition_key, snapshot)
        .await;
}
