use my_azure_storage_sdk::{blob::BlobApi, AzureConnectionWithTelemetry, AzureStorageError};

use crate::telemetry::TelemetryWriter;

pub async fn delete(
    azure_connection: &AzureConnectionWithTelemetry<TelemetryWriter>,
    table_name: &str,
    partition_key: &str,
) -> Result<(), AzureStorageError> {
    let blob_name = super::utils::get_blob_file_name_by_partition_name(partition_key);
    azure_connection
        .delete_blob_if_exists(table_name, blob_name.as_str())
        .await?;

    Ok(())
}
