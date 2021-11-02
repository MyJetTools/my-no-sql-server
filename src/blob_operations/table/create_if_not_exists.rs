use my_azure_storage_sdk::AzureConnection;
use my_azure_storage_sdk::AzureStorageError;
use my_azure_storage_sdk::BlobContainersApi;

pub async fn create_if_not_exists(
    azure_connection: &AzureConnection,
    table_name: &str,
) -> Result<(), AzureStorageError> {
    azure_connection
        .create_container_if_not_exist(table_name)
        .await
}
