use std::sync::Arc;

use my_azure_storage_sdk::AzureConnection;
use my_azure_storage_sdk::AzureStorageError;
use my_azure_storage_sdk::BlobApi;
use my_azure_storage_sdk::BlobContainersApi;

use my_azure_storage_sdk::BlockBlobApi;
use serde::{Deserialize, Serialize};

use crate::app::AppServices;
use crate::db::DbPartition;
use crate::db::DbRow;
use crate::db::{DbTableAttributes, DbTableData};

const METADATA_BLOB_NAME: &str = ".metadata";

#[derive(Serialize, Deserialize, Debug)]
pub struct TableMetadataFileContract {
    #[serde(rename = "Persist")]
    #[serde(default = "default_persist")]
    pub persist: bool,
    #[serde(rename = "MaxPartitionsAmount")]
    pub max_partitions_amount: Option<usize>,
}

fn default_persist() -> bool {
    true
}

pub async fn get_tables(
    azure_connection: &AzureConnection,
) -> Result<Vec<String>, AzureStorageError> {
    let containers = azure_connection.get_list_of_blob_containers().await?;

    Ok(containers)
}

pub async fn load_table(
    azure_connection: &AzureConnection,
    table_name: &str,
) -> Result<DbTableData, AzureStorageError> {
    let blobs = azure_connection.get_list_of_blobs(table_name).await?;

    let attributes = DbTableAttributes {
        max_partitions_amount: None,
        persist: true,
    };

    let mut db_table_data = DbTableData::new(attributes);

    for blob_name in blobs {
        let raw = azure_connection
            .download_blob(table_name, blob_name.as_str())
            .await?;

        if blob_name == METADATA_BLOB_NAME {
            let table_metadata = parse_table_metadata(raw.as_slice());

            db_table_data.attributes.max_partitions_amount = table_metadata.max_partitions_amount;
            db_table_data.attributes.persist = table_metadata.persist;
        } else {
            let partition_name = base64::decode(blob_name.as_str()).unwrap();
            let partition_key = String::from_utf8(partition_name).unwrap();
            println!("Initializing partition: {}. ", partition_key,);

            let entities = crate::json::array_parser::split_to_objects(raw.as_slice()).unwrap();

            let mut db_partition = DbPartition::new();

            for entity in &entities {
                let db_row = DbRow::form_db_entity(entity);
                db_partition
                    .rows
                    .insert(db_row.row_key.to_string(), Arc::new(db_row));
            }

            db_table_data.init_partition(partition_key, db_partition);
        }
    }

    return Ok(db_table_data);
}

pub async fn clean_table(
    azure_connection: &AzureConnection,
    table_name: &str,
) -> Result<(), AzureStorageError> {
    let blobs = azure_connection.get_list_of_blobs(table_name).await?;

    for blob_name in blobs {
        if blob_name != METADATA_BLOB_NAME {
            azure_connection
                .delete_blob(table_name, blob_name.as_str())
                .await?;
        }
    }

    Ok(())
}

pub async fn delete_partition(
    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
) -> Result<(), AzureStorageError> {
    let blob_name = get_blob_file_by_partition_name(partition_key);
    azure_connection
        .delete_blob(table_name, blob_name.as_str())
        .await?;

    Ok(())
}

pub async fn save_partition(
    azure_connection: &AzureConnection,
    table_name: &str,
    partition_key: &str,
    content: Vec<u8>,
) -> Result<(), AzureStorageError> {
    let blob_file = get_blob_file_by_partition_name(partition_key);

    azure_connection
        .upload(table_name, blob_file.as_str(), content)
        .await?;

    Ok(())
}

fn get_blob_file_by_partition_name(partition_name: &str) -> String {
    base64::encode(partition_name.as_bytes())
}

fn parse_table_metadata(content: &[u8]) -> TableMetadataFileContract {
    let parse_result = serde_json::from_slice::<TableMetadataFileContract>(content);

    match parse_result {
        Ok(res) => res,
        Err(_) => TableMetadataFileContract {
            max_partitions_amount: None,
            persist: true,
        },
    }
}

pub async fn save_table_attributes(
    azure_connection: &AzureConnection,
    table_name: &str,
    attributes: &DbTableAttributes,
    app: &AppServices,
) -> Result<(), AzureStorageError> {
    let contract = TableMetadataFileContract {
        persist: attributes.persist,
        max_partitions_amount: attributes.max_partitions_amount,
    };

    let serialize_result = serde_json::to_vec(&contract);

    match serialize_result {
        Ok(json) => {
            azure_connection
                .upload(table_name, METADATA_BLOB_NAME, json)
                .await?;

            return Ok(());
        }
        Err(err) => {
            let msg = format!(
                "Could not serialize table attributes to save it to the table. {}",
                err
            );
            app.logs
                .add_info(
                    Some(table_name.to_string()),
                    "save_table_attributes".to_string(),
                    msg.to_string(),
                )
                .await;

            return Err(AzureStorageError::UnknownError { msg });
        }
    };
}
