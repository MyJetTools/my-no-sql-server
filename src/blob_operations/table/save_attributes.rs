use my_azure_storage_sdk::{
    block_blob::BlockBlobApi, AzureConnectionWithTelemetry, AzureStorageError,
};

use crate::{db::DbTableAttributes, telemetry::TelemetryWriter};

use super::metadata::TableMetadataFileContract;

pub async fn save_attributes(
    azure_connection: &AzureConnectionWithTelemetry<TelemetryWriter>,
    table_name: &str,
    attributes: &DbTableAttributes,
) -> Result<(), AzureStorageError> {
    let contract = TableMetadataFileContract {
        persist: attributes.persist,
        max_partitions_amount: attributes.max_partitions_amount,
    };

    let serialize_result = serde_json::to_vec(&contract);

    match serialize_result {
        Ok(json) => {
            azure_connection
                .upload(table_name, super::metadata::METADATA_BLOB_NAME, json)
                .await?;

            return Ok(());
        }
        Err(err) => {
            let msg = format!(
                "Could not serialize table attributes to save it to the table. {}",
                err
            );

            return Err(AzureStorageError::UnknownError { msg });
        }
    };
}
