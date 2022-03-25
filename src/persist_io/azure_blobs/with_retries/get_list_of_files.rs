use my_azure_storage_sdk::{
    blob_container::BlobContainersApi, sdk_azure::blobs::AzureBlobsListReader,
    AzureStorageConnection, AzureStorageConnectionData,
};

use crate::{app::logs::Logs, persist_io::TableFile};

pub async fn get_list_of_files(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
) -> Vec<TableFile> {
    let file_names = match azure_connection {
        AzureStorageConnection::AzureStorage(connection_data) => {
            get_list_of_files_from_azure_blob_container(logs, connection_data, table_name).await
        }
        _ => azure_connection
            .get_list_of_blobs(table_name)
            .await
            .unwrap(),
    };

    let mut result = Vec::new();

    for file_name in file_names {
        result.push(TableFile::from_file_name(file_name.as_str()).unwrap())
    }

    result
}

async fn get_list_of_files_from_azure_blob_container(
    logs: &Logs,
    connection: &AzureStorageConnectionData,
    table_name: &str,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut attempt_no: u8 = 0;
    let mut blobs_list_reader = AzureBlobsListReader::new(connection, table_name);
    loop {
        let next_result = blobs_list_reader.get_next().await;
        match next_result {
            Ok(chunk) => {
                if let Some(chunk) = chunk {
                    result.extend(chunk)
                } else {
                    return result;
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
