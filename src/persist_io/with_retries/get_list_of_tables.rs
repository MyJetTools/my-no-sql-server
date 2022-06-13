use my_azure_storage_sdk::blob_container::BlobContainersApi;
use my_azure_storage_sdk::AzureStorageConnectionData;

use my_azure_storage_sdk::sdk_azure::containers::AzureContainersListReader;
use my_azure_storage_sdk::AzureStorageConnection;

use crate::app::logs::Logs;

pub async fn get_list_of_tables(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
) -> Vec<String> {
    match azure_connection {
        AzureStorageConnection::AzureStorage(connection_data) => {
            return get_list_of_tables_from_azure_blob_container(logs, connection_data).await
        }
        _ => azure_connection
            .get_list_of_blob_containers()
            .await
            .unwrap(),
    }
}

async fn get_list_of_tables_from_azure_blob_container(
    logs: &Logs,
    connection: &AzureStorageConnectionData,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut attempt_no: u8 = 0;
    let mut blobs_list_reader = AzureContainersListReader::new(connection);
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
                    None,
                    "get_list_of_tables_from_azure_blob_container",
                    format!("Can not get list of tables. Err: {:?}", err),
                    attempt_no,
                )
                .await;

                attempt_no += 1;
            }
        }
    }
}
