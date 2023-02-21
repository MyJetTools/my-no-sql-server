use my_azure_storage_sdk::{
    blob_container::BlobContainersApi, sdk_azure::blobs::AzureBlobsListReader,
    AzureStorageConnection, AzureStorageConnectionData,
};

use crate::persist_io::persist_io_operations::TableListOfFilesUploader;
use my_no_sql_server_core::logs::*;

pub async fn get_list_of_files<TTableListOfFilesUploader: TableListOfFilesUploader>(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    uploader: &TTableListOfFilesUploader,
) {
    match azure_connection {
        AzureStorageConnection::AzureStorage(connection_data) => {
            get_list_of_files_from_azure_blob_container(
                logs,
                connection_data,
                table_name,
                uploader,
            )
            .await;
        }
        _ => {
            let chunk = azure_connection
                .get_list_of_blobs(table_name)
                .await
                .unwrap();

            uploader.add_files(table_name, chunk).await;
            uploader.set_files_list_is_loaded(table_name).await;
        }
    };
}

async fn get_list_of_files_from_azure_blob_container<
    TTableListOfFilesUploader: TableListOfFilesUploader,
>(
    logs: &Logs,
    connection: &AzureStorageConnectionData,
    table_name: &str,
    uploader: &TTableListOfFilesUploader,
) {
    let mut attempt_no: u8 = 0;
    let mut blobs_list_reader = AzureBlobsListReader::new(connection, table_name);
    loop {
        let next_result = blobs_list_reader.get_next().await;
        match next_result {
            Ok(chunk) => {
                if let Some(chunk) = chunk {
                    uploader.add_files(table_name, chunk).await;
                } else {
                    uploader.set_files_list_is_loaded(table_name).await;
                    return;
                }
            }
            Err(err) => {
                super::attempt_handling::execute(
                    logs,
                    Some(table_name.to_string()),
                    "get_list_of_files_from_azure_blob_container",
                    format!(
                        "Can not get list of files from azure blob container:[{}]. Err: {:?}",
                        table_name, err
                    ),
                    attempt_no,
                )
                .await;
                attempt_no += 1;
            }
        }
    }
}
