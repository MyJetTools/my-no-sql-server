use my_azure_storage_sdk::{blob::BlobApi, AzureStorageConnection, AzureStorageError};

pub async fn load_table_file(
    azure_connection: &AzureStorageConnection,
    table_name: &str,
    blob_file: &str,
) -> Option<Vec<u8>> {
    let mut attempt_no = 0;

    loop {
        let result = azure_connection.download_blob(table_name, blob_file).await;

        match result {
            Ok(result) => {
                return Some(result);
            }
            Err(err) => {
                if let AzureStorageError::BlobNotFound = &err {
                    return None;
                }

                super::attempt_handling::execute(
                    Some(table_name),
                    "load_table_file",
                    format!(
                        "Can not download blob {}/{}. Err: {:?}",
                        table_name, blob_file, err
                    ),
                    attempt_no,
                )
                .await;

                if let AzureStorageError::InvalidResourceName = &err {
                    panic!(
                        "Can not download blob {}/{}. Reason: {:?}",
                        table_name, blob_file, err
                    )
                }
                attempt_no += 1;
            }
        }
    }
}
