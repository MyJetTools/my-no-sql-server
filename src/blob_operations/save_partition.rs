use std::time::Duration;

use my_azure_storage_sdk::AzureConnection;

use crate::{
    app::{logs::SystemProcess, AppContext},
    db::{read_as_json::DbEntityAsJsonArray, DbPartitionSnapshot},
};

pub async fn with_retries(
    app: &AppContext,
    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
    snapshot: DbPartitionSnapshot,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::partition::save(
            azure_connection,
            table_name,
            partition_key,
            snapshot.content.as_json_array(),
        )
        .await;

        if result.is_ok() {
            app.blob_content_cache
                .update_table_partition_snapshot_id(
                    table_name,
                    partition_key,
                    snapshot.last_write_moment,
                )
                .await;

            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "save_partition".to_string(),
                    "Saved".to_string(),
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
