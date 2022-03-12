use my_azure_storage_sdk::{block_blob::BlockBlobApi, AzureStorageConnection, AzureStorageError};

pub async fn save(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    partition_key: &str,
    content: Vec<u8>,
) -> Result<(), AzureStorageError> {
    let blob_file = super::super::super::serializers::blob_file_name::encode(partition_key);

    azure_connection
        .upload(table_name, blob_file.as_str(), content)
        .await
}
