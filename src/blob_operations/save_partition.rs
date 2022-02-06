use std::time::Duration;

use my_azure_storage_sdk::AzureStorageConnection;
use rust_extensions::StopWatch;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::db_snapshots::DbPartitionSnapshot,
};

pub async fn with_retries(
    app: &AppContext,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    partition_key: &str,
    db_partition_snapshot: &DbPartitionSnapshot,
) {
    let mut attempt_no = 0;
    let mut stop_watch = StopWatch::new();
    stop_watch.start();
    loop {
        let result = super::partition::save(
            azure_connection,
            table_name,
            partition_key,
            db_partition_snapshot.db_rows.as_json_array().build(),
        )
        .await;

        if result.is_ok() {
            app.blob_content_cache
                .update_table_partition_snapshot_id(
                    table_name,
                    partition_key,
                    db_partition_snapshot.last_write_moment,
                )
                .await;

            stop_watch.pause();

            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "save_partition".to_string(),
                    format!(
                        "Saved partition {}/{} in {}",
                        table_name,
                        partition_key,
                        stop_watch.duration_as_string()
                    ),
                )
                .await;
            return;
        }

        let err = result.err().unwrap();

        attempt_no += 1;

        app.logs
            .add_error(
                Some(table_name.to_string()),
                SystemProcess::BlobOperation,
                "save_partition".to_string(),
                format!(
                    "Can not sync partition {}. Attempt: {}",
                    partition_key, attempt_no
                ),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
