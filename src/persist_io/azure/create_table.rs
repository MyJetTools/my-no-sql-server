use my_azure_storage_sdk::{blob_container::BlobContainersApi, AzureStorageConnection};

pub async fn create_table(azure_connection: &AzureStorageConnection, table_name: &str) {
    let mut attempt_no = 0;
    while let Err(err) = azure_connection
        .create_container_if_not_exists(table_name)
        .await
    {
        super::attempt_handling::execute(
            Some(table_name),
            "create_table",
            format!("Error creating table [{}]. Err: {:?}", table_name, err),
            attempt_no,
        )
        .await;

        attempt_no += 1;
    }
}
