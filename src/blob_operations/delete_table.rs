use std::time::Duration;

use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;

use crate::app::AppContext;

use my_app_insights::AppInsightsTelemetry;

pub async fn with_retries(
    app: &AppContext,
    azure_connection: &AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>,
    table_name: &str,
) {
    let mut attempt_no = 0;
    loop {
        let result = super::table::delete(azure_connection, table_name).await;

        if result.is_ok() {
            app.blob_content_cache.delete_table(table_name).await;

            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "delete_table".to_string(),
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
                crate::app::logs::SystemProcess::BlobOperation,
                "delete_table".to_string(),
                format!("Attempt: {}", attempt_no),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}
