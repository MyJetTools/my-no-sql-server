use my_azure_storage_sdk::AzureConnection;
use my_azure_storage_sdk::AzureStorageError;
use my_azure_storage_sdk::BlobContainersApi;

pub async fn get_list(
    azure_connection: &AzureConnection,
) -> Result<Vec<String>, AzureStorageError> {
    let containers = azure_connection.get_list_of_blob_containers().await?;

    Ok(containers)
}
