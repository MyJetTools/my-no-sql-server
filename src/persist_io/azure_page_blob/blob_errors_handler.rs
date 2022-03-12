use my_azure_storage_sdk::{
    blob_container::BlobContainersApi, AzureStorageConnection, AzureStorageError,
};

use crate::app::logs::{Logs, SystemProcess};

pub async fn handle_azure_blob_error(
    logs: &Logs,
    process_name: &str,
    err: &AzureStorageError,
    table_name: &str,
    azure_connection: &AzureStorageConnection,
    attempt_no: usize,
) {
    logs.add_error(
        Some(table_name.to_string()),
        SystemProcess::PersistOperation,
        process_name.to_string(),
        format!("Azure storage error with table:{table_name}. Attempt: {attempt_no}"),
        Some(format!("{:?}", err)),
    );
    match err {
        AzureStorageError::ContainerNotFound => {
            create_table_container(logs, table_name, azure_connection).await;
        }
        AzureStorageError::BlobNotFound => {}
        AzureStorageError::BlobAlreadyExists => {}
        AzureStorageError::ContainerBeingDeleted => {}
        AzureStorageError::ContainerAlreadyExists => {}
        AzureStorageError::InvalidPageRange => {}
        AzureStorageError::RequestBodyTooLarge => {}
        AzureStorageError::UnknownError { msg } => {
            println!("handle_azure_blob_error::Unknown error:{} ", msg);
        }
        AzureStorageError::HyperError { err } => {
            println!("handle_azure_blob_error::HyperError:{:?} ", err);
        }
    }
}

async fn create_table_container(
    logs: &Logs,
    table_name: &str,
    azure_connection: &AzureStorageConnection,
) {
    if let Err(err) = azure_connection
        .create_container_if_not_exist(table_name)
        .await
    {
        logs.add_error(
            Some(table_name.to_string()),
            SystemProcess::PersistOperation,
            "create_table_container".to_string(),
            format!("Azure storage error with table: {table_name}"),
            Some(format!("{:?}", err)),
        );
    }
}
