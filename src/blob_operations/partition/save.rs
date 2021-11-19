use my_azure_storage_sdk::{
    block_blob::BlockBlobApi, AzureConnectionWithTelemetry, AzureStorageError,
};

use my_app_insights::AppInsightsTelemetry;

pub async fn save(
    azure_connection: &AzureConnectionWithTelemetry<AppInsightsTelemetry>,
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
