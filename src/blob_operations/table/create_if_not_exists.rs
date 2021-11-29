use my_azure_storage_sdk::blob_container::BlobContainersApi;
use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;
use my_azure_storage_sdk::AzureStorageError;

use my_app_insights::AppInsightsTelemetry;

pub async fn create_if_not_exists(
    azure_connection: &AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>,
    table_name: &str,
) -> Result<(), AzureStorageError> {
    azure_connection
        .create_container_if_not_exist(table_name)
        .await
}
