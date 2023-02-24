use my_no_sql_server_core::db_snapshots::DbPartitionSnapshot;

use crate::{app::AppContext, persist_io::TableFile};

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
