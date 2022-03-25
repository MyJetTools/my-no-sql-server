use my_azure_storage_sdk::{block_blob::BlockBlobApi, AzureStorageConnection, AzureStorageError};

use crate::db::DbTableAttributesSnapshot;
//TODO - Delete
pub async fn save_attributes(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    attributes: &DbTableAttributesSnapshot,
) -> Result<(), AzureStorageError> {
    let contract = table_attrs::TableMetadataFileContract {
        persist: attributes.persist,
        max_partitions_amount: attributes.max_partitions_amount,
    };

    let serialize_result = serde_json::to_vec(&contract);

    match serialize_result {
        Ok(json) => {
            azure_connection
                .upload(table_name, table_attrs::METADATA_FILE_NAME, json)
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
