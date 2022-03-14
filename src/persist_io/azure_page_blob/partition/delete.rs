use my_azure_storage_sdk::{blob::BlobApi, AzureStorageConnection, AzureStorageError};

pub async fn delete(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    partition_key: &str,
) -> Result<(), AzureStorageError> {
    let blob_name = super::super::super::serializers::blob_file_name::encode(partition_key);
    azure_connection
        .delete_blob_if_exists(table_name, blob_name.as_str())
        .await?;

    Ok(())
}
