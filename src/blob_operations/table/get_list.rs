use my_azure_storage_sdk::blob_container::BlobContainersApi;

use my_azure_storage_sdk::AzureConnectionWithTelemetry;
use my_azure_storage_sdk::AzureStorageError;

use my_app_insights::AppInsightsTelemetry;

pub async fn get_list(
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
) -> Result<Vec<String>, AzureStorageError> {
    let containers = azure_connection.get_list_of_blob_containers().await?;

    Ok(containers)
}
