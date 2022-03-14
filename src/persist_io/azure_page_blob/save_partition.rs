use std::time::Duration;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::{app::logs::Logs, db::db_snapshots::DbPartitionSnapshot};

pub async fn with_retries(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    partition_key: &str,
    db_partition_snapshot: &DbPartitionSnapshot,
) {
    let mut attempt_no = 0;

    loop {
        match super::partition::save(
            azure_connection,
            table_name,
            partition_key,
            db_partition_snapshot
                .db_rows_snapshot
                .as_json_array()
                .build(),
        )
        .await
        {
            Ok(_) => {
                return;
            }
            Err(err) => {
                attempt_no += 1;

                super::blob_errors_handler::handle_azure_blob_error(
                    logs,
                    "save_partition",
                    &err,
                    table_name,
                    azure_connection,
                    attempt_no,
                )
                .await;

                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }
}
