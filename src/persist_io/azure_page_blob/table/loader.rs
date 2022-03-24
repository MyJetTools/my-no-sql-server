use std::sync::Arc;
use std::time::Duration;

use my_azure_storage_sdk::blob::BlobApi;
use my_azure_storage_sdk::AzureStorageConnection;

use my_azure_storage_sdk::blob_container::BlobContainersApi;
use my_azure_storage_sdk::sdk_azure::blobs::AzureBlobsListReader;
use tokio::sync::Mutex;

use crate::persist_io::serializers::table_attrs::{TableMetadataFileContract, METADATA_FILE_NAME};
use crate::persist_io::{serializers, TableLoadItem};

pub struct PageBlobTableLoader {
    blobs: Mutex<Vec<String>>,
    azure_connection: Arc<AzureStorageConnection>,
    table_name: String,
}

impl PageBlobTableLoader {
    pub async fn new(azure_connection: Arc<AzureStorageConnection>, table_name: &str) -> Self {
        let result = get_list_of_blobs(azure_connection.as_ref(), table_name).await;

        Self {
            blobs: Mutex::new(result),
            azure_connection,
            table_name: table_name.to_string(),
        }
    }

    async fn get_next_blob_name(&self) -> Option<String> {
        let mut access = self.blobs.lock().await;
        if access.len() == 0 {
            return None;
        }

        let blob_name = access.remove(0);
        Some(blob_name)
    }

    pub async fn get_next(&self) -> Option<TableLoadItem> {
        let blob_name = self.get_next_blob_name().await?;

        let raw = download_with_retries(
            self.azure_connection.as_ref(),
            &self.table_name,
            blob_name.as_str(),
        )
        .await;

        if blob_name == METADATA_FILE_NAME {
            let table_metadata = TableMetadataFileContract::parse(raw.as_slice());

            return TableLoadItem::TableAttributes(table_metadata.into()).into();
        }

        let partition_key = serializers::blob_file_name::decode(blob_name.as_str());

        TableLoadItem::DbPartition {
            partition_key,
            db_partition: serializers::db_partition::deserialize(raw.as_slice()),
        }
        .into()
    }
}

async fn get_list_of_blobs(
    azure_connection: &AzureStorageConnection,
    container_name: &str,
) -> Vec<String> {
    match azure_connection {
        AzureStorageConnection::AzureStorage(connection_data) => {
            let reader = AzureBlobsListReader::new(connection_data, container_name);

            let reader = Mutex::new(reader);

            let mut result = Vec::new();

            while let Some(chunk) = get_next_with_retries(&reader, container_name).await {
                result.extend(chunk);
            }

            result
        }
        _ => azure_connection
            .get_list_of_blobs(container_name)
            .await
            .unwrap(),
    }
}

async fn get_next_with_retries<'s>(
    reader: &'s Mutex<AzureBlobsListReader<'s>>,
    container_name: &str,
) -> Option<Vec<String>> {
    let mut attempt_no: u8 = 0;
    loop {
        let mut write_access = reader.lock().await;
        match write_access.get_next().await {
            Ok(result) => return result,
            Err(err) => {
                if attempt_no == 5 {
                    panic!(
                        "Can not get list of blobs for container: {}",
                        container_name
                    );
                }

                println!(
                    "Attempt:[{}]. Can not get list of blobs for container: {}. Retrying... Err:{:?}",
                    attempt_no, container_name, err
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempt_no += 1;
            }
        }
    }
}

async fn download_with_retries(
    azure_connection: &AzureStorageConnection,
    container_name: &str,
    blob_name: &str,
) -> Vec<u8> {
    let mut attempt_no: u8 = 0;

    loop {
        let result = azure_connection
            .download_blob(container_name, blob_name)
            .await;

        match result {
            Ok(result) => return result,
            Err(err) => {
                if attempt_no == 5 {
                    panic!(
                        "Can not get list of blobs for container: {}",
                        container_name
                    );
                }

                println!(
                    "Attempt:[{}]. Can not get list of blobs for container: {}. Retrying... Err:{:?}",
                    attempt_no, container_name, err
                );
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempt_no += 1;
            }
        }
    }
}
