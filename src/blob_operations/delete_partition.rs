use std::time::Duration;

use my_azure_storage_sdk::AzureConnectionWithTelemetry;

use crate::app::{logs::SystemProcess, AppContext};

use my_app_insights::AppInsightsTelemetry;

pub async fn with_retires(
    app: &AppContext,
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
    table_name: &str,
    partition_key: &str,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::partition::delete(azure_connection, table_name, partition_key).await;

        if result.is_ok() {
            app.blob_content_cache
                .delete_table_partition(table_name, partition_key)
                .await;

            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "delete_partition".to_string(),
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
                "delete_partition".to_string(),
                format!("PartitionKey: {}, Attempt: {}", partition_key, attempt_no),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
