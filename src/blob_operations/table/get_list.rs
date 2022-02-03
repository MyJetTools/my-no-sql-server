use my_azure_storage_sdk::blob_container::BlobContainersApi;

use my_azure_storage_sdk::AzureStorageConnection;
use my_azure_storage_sdk::AzureStorageError;

pub async fn get_list(
    azure_connection: &AzureStorageConnection,
) -> Result<Vec<String>, AzureStorageError> {
    let containers = azure_connection.get_list_of_blob_containers().await?;

    Ok(containers)
}
