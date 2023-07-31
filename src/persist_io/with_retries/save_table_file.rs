use my_azure_storage_sdk::{block_blob::BlockBlobApi, AzureStorageConnection};

use my_no_sql_server_core::logs::*;

pub async fn save_table_file(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    container_name: &str,
    blob_name: &str,
    content: Vec<u8>,
) {
    let mut attempt_no = 0;

    while let Err(err) = azure_connection
        .upload_block_blob(container_name, blob_name, content.to_vec())
        .await
    {
        super::attempt_handling::execute(
            logs,
            Some(container_name.to_string()),
            "save_table_file",
            format!(
                "Can not save blob {}/{}. Err: {:?}",
                container_name, blob_name, err
            ),
            attempt_no,
        )
        .await;

        attempt_no += 1;
    }
}
