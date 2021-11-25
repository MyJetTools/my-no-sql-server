use std::time::Duration;

use my_azure_storage_sdk::{AzureConnectionWithTelemetry, AzureStorageError};

use crate::{app::AppContext, db::DbTableAttributesSnapshot};
use my_app_insights::AppInsightsTelemetry;

pub async fn with_retries(
    app: &AppContext,
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
    table_name: &str,
    attr: &DbTableAttributesSnapshot,
) {
    let mut attempt_no = 0;
    loop {
        let result = create_table(azure_connection, table_name, attr).await;

        if result.is_ok() {
            app.blob_content_cache
                .create_table(table_name, attr.clone())
                .await;

            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::BlobOperation,
                    "create_table".to_string(),
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
                "create_table".to_string(),
                format!("Attempt: {}", attempt_no),
                Some(format!("{:?}", err)),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn create_table(
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
    table_name: &str,
    attr: &DbTableAttributesSnapshot,
) -> Result<(), AzureStorageError> {
    super::table::create_if_not_exists(azure_connection, table_name).await?;
    super::table::save_attributes(azure_connection, table_name, attr).await?;

    Ok(())
}
