use my_azure_storage_sdk::blob::BlobApi;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::app::logs::Logs;

pub async fn delete_table_file(
    logs: &Logs,
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    blob_name: &str,
) {
    let mut attempt_no = 0;

    while let Err(err) = azure_connection
        .delete_blob_if_exists(table_name, blob_name)
        .await
    {
        super::attempt_handling::execute(
            logs,
            Some(table_name.to_string()),
            "delete_table_file",
            format!(
                "Can not delete blob file: {}/{}. Err: {:?}",
                table_name, blob_name, err
            ),
            attempt_no,
        )
        .await;
        attempt_no += 1;
    }
}
