use my_azure_storage_sdk::{block_blob::BlockBlobApi, AzureStorageConnection};

pub async fn save_table_file(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    blob_name: &str,
    content: Vec<u8>,
) {
    let mut attempt_no = 0;

    while let Err(err) = azure_connection
        .upload_block_blob(table_name, blob_name, content.to_vec())
        .await
    {
        super::attempt_handling::execute(
            Some(table_name),
            "save_table_file",
            format!(
                "Can not save blob {}/{}. Err: {:?}",
                table_name, blob_name, err
            ),
            attempt_no,
        )
        .await;

        attempt_no += 1;
    }
}
