use my_azure_storage_sdk::blob_container::BlobContainersApi;

use my_azure_storage_sdk::AzureStorageConnectionWithTelemetry;
use my_azure_storage_sdk::AzureStorageError;

use my_app_insights::AppInsightsTelemetry;

pub async fn get_list(
    azure_connection: &AzureStorageConnectionWithTelemetry<AppInsightsTelemetry>,
) -> Result<Vec<String>, AzureStorageError> {
    let containers = azure_connection.get_list_of_blob_containers().await?;

    Ok(containers)
}
