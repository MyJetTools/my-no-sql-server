use my_azure_storage_sdk::blob::BlobApi;
use my_azure_storage_sdk::blob_container::BlobContainersApi;

use my_azure_storage_sdk::{AzureConnectionWithTelemetry, AzureStorageError};

use crate::telemetry::TelemetryWriter;

pub async fn delete(
    azure_connection: &AzureConnectionWithTelemetry<TelemetryWriter>,
    table_name: &str,
) -> Result<(), AzureStorageError> {
    let blobs = azure_connection.get_list_of_blobs(table_name).await;

    if let Err(AzureStorageError::ContainerNotFound) = blobs {
        return Ok(());
    }

    for blob_name in blobs? {
        azure_connection
            .delete_blob_if_exists(table_name, blob_name.as_str())
            .await?;
    }

    azure_connection
        .delete_container_if_exists(table_name)
        .await?;

    Ok(())
}
