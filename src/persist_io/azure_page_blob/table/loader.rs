use std::sync::Arc;

use my_azure_storage_sdk::blob::BlobApi;
use my_azure_storage_sdk::AzureStorageConnection;

use my_azure_storage_sdk::blob_container::BlobContainersApi;
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
        let blobs = azure_connection
            .get_list_of_blobs(&table_name)
            .await
            .unwrap();

        Self {
            blobs: Mutex::new(blobs),
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

        let raw = self
            .azure_connection
            .download_blob(&self.table_name, blob_name.as_str())
            .await
            .unwrap();

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

/*
async fn init_to_db_table(
    db_table_data: &mut DbTableData,
    table_attributes: &mut DbTableAttributesSnapshot,
    tasks: &mut Vec<JoinHandle<LoadBlobResult>>,
) {
    for task in tasks.drain(..) {
        match task.await {
            Ok(result) => match result {
                LoadBlobResult::Metadata(meta_data) => {
                    table_attributes.max_partitions_amount = meta_data.max_partitions_amount;
                    table_attributes.persist = meta_data.persist;
                }
                LoadBlobResult::DbPartition {
                    partition_key,
                    db_partition,
                } => {
                    db_table_data.init_partition(partition_key, db_partition);
                }
            },
            Err(_) => {
                println!(
                    "Error loading partition for table {}. Skipping partition",
                    db_table_data.name
                );
            }
        }
    }
}

pub enum LoadBlobResult {
    Metadata(TableMetadataFileContract),
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}
 */
