use std::sync::Arc;

use my_azure_storage_sdk::{
    blob_container::BlobContainersApi, sdk_azure::blobs::AzureBlobsListReader,
    AzureStorageConnection, AzureStorageConnectionData,
};

use crate::{app::logs::Logs, persist_operations::data_initializer::load_tasks::TableToLoad};

pub async fn get_list_of_files(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_to_load: &Arc<TableToLoad>,
) {
    match azure_connection {
        AzureStorageConnection::AzureStorage(connection_data) => {
            get_list_of_files_from_azure_blob_container(logs, connection_data, table_to_load).await;
        }
        _ => {
            let chunk = azure_connection
                .get_list_of_blobs(table_to_load.table_name.as_str())
                .await
                .unwrap();

            table_to_load.add_partitions_to_load(chunk).await;
            table_to_load.set_files_list_is_loaded();
        }
    };
}

async fn get_list_of_files_from_azure_blob_container(
    logs: &Logs,
    connection: &AzureStorageConnectionData,
    table_to_load: &Arc<TableToLoad>,
) {
    let mut attempt_no: u8 = 0;
    let mut blobs_list_reader =
        AzureBlobsListReader::new(connection, table_to_load.table_name.as_str());
    loop {
        let next_result = blobs_list_reader.get_next().await;
        match next_result {
            Ok(chunk) => {
                if let Some(chunk) = chunk {
                    table_to_load.add_partitions_to_load(chunk).await;
                } else {
                    table_to_load.set_files_list_is_loaded();
                    return;
                }
            }
            Err(err) => {
                super::attempt_handling::execute(
                    logs,
                    Some(table_to_load.table_name.to_string()),
                    "get_list_of_files_from_azure_blob_container",
                    format!(
                        "Can not get list of files from azure blob container:[{}]. Err: {:?}",
                        table_to_load.table_name, err
                    ),
                    attempt_no,
                )
                .await;
                attempt_no += 1;
            }
        }
    }
}
