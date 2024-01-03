use my_azure_storage_sdk::{blob_container::BlobContainersApi, AzureStorageConnection};

pub async fn delete_table_folder(azure_connection: &AzureStorageConnection, table_name: &str) {
    let mut attempt_no = 0;
    while let Err(err) = azure_connection
        .delete_container_if_exists(table_name)
        .await
    {
        super::attempt_handling::execute(
            Some(table_name),
            "delete_container",
            format!("Can not delete container: {}. Err: {:?}", table_name, err),
            attempt_no,
        )
        .await;
        attempt_no += 1;
    }
}
