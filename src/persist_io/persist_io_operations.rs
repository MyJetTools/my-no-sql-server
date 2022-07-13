use std::sync::Arc;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::{app::logs::Logs, persist_io::TableFile};

pub struct PersistIoOperations {
    logs: Arc<Logs>,
    azure_connection: Arc<AzureStorageConnection>,
}

#[async_trait::async_trait]
pub trait TableListOfFilesUploader {
    async fn add_files(&self, table_name: &str, files: Vec<String>);
    async fn set_files_list_is_loaded(&self, table_name: &str);
}

impl PersistIoOperations {
    pub fn new(azure_connection: Arc<AzureStorageConnection>, logs: Arc<Logs>) -> Self {
        Self {
            logs,
            azure_connection,
        }
    }

    pub async fn get_list_of_tables(&self) -> Vec<String> {
        super::with_retries::get_list_of_tables(self.logs.as_ref(), self.azure_connection.as_ref())
            .await
    }

    pub async fn get_table_files<TTableListOfFilesUploader: TableListOfFilesUploader>(
        &self,
        table_name: &str,
        uploader: &TTableListOfFilesUploader,
    ) {
        super::with_retries::get_list_of_files(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
            uploader,
        )
        .await;
    }

    pub async fn create_table_folder(&self, table_name: &str) {
        super::with_retries::create_table(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
        )
        .await;
    }

    pub async fn save_table_file(
        &self,
        table_name: &str,
        table_file: &TableFile,
        content: Vec<u8>,
    ) {
        super::with_retries::save_table_file(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
            table_file.get_file_name().as_str(),
            content,
        )
        .await;
    }

    pub async fn delete_table_file(&self, table_name: &str, table_file: &TableFile) {
        super::with_retries::delete_table_file(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
            table_file.get_file_name().as_str(),
        )
        .await;
    }

    pub async fn delete_table_folder(&self, table_name: &str) {
        super::with_retries::delete_table_folder(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
        )
        .await;
    }

    pub async fn load_table_file(
        &self,
        table_name: &str,
        table_file: &TableFile,
    ) -> Option<Vec<u8>> {
        super::with_retries::load_table_file(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
            table_file.get_file_name().as_str(),
        )
        .await
    }
}
