use my_azure_storage_sdk::{AzureConnection, AzureStorageError, BlockBlobApi};

pub async fn save(
    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
    content: Vec<u8>,
) -> Result<(), AzureStorageError> {
    let blob_file = super::utils::get_blob_file_name_by_partition_name(partition_key);

    azure_connection
        .upload(table_name, blob_file.as_str(), content)
        .await?;

    Ok(())
}
