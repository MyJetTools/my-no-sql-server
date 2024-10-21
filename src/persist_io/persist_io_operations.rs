use std::sync::Arc;

use my_azure_storage_sdk::AzureStorageConnection;

use my_no_sql_server_core::db_snapshots::DbTableSnapshot;
use tokio::sync::Mutex;

use crate::{persist_io::TableFile, sqlite_repo::SqlLiteRepo};

use super::{sqlite::InitContainer, TABLE_METADATA_FILE_NAME};

pub enum PersistIoOperations {
    AzureConnection(Arc<AzureStorageConnection>),
    SqLite {
        repo: SqlLiteRepo,
        init_container: Mutex<InitContainer>,
    },
}

#[async_trait::async_trait]
pub trait TableListOfFilesUploader {
    async fn add_files(&self, table_name: &str, files: Vec<String>);
    async fn set_files_list_is_loaded(&self, table_name: &str);
}

impl PersistIoOperations {
    pub fn as_azure_connection(azure_connection: Arc<AzureStorageConnection>) -> Self {
        Self::AzureConnection(azure_connection)
    }

    pub fn as_sqlite(repo: SqlLiteRepo) -> Self {
        Self::SqLite {
            repo,
            init_container: Mutex::new(InitContainer::new()),
        }
    }

    pub async fn get_list_of_tables(&self) -> Vec<String> {
        match self {
            Self::SqLite {
                repo,
                init_container,
            } => {
                let tables = repo.get_files().await;
                let mut read_access = init_container.lock().await;
                read_access.init(tables);
                read_access.get_list_of_tables()
            }
            Self::AzureConnection(azure_connection) => {
                super::azure::get_list_of_tables(azure_connection.as_ref()).await
            }
        }
    }

    pub async fn get_table_files<TTableListOfFilesUploader: TableListOfFilesUploader>(
        &self,
        table_name: &str,
        uploader: &TTableListOfFilesUploader,
    ) {
        match self {
            Self::SqLite {
                repo: _,
                init_container,
            } => {
                let init_container = init_container.lock().await;
                let files = init_container.get_file_names(table_name);
                uploader.add_files(table_name, files).await;
                uploader.set_files_list_is_loaded(table_name).await;
            }
            Self::AzureConnection(azure_connection) => {
                super::azure::get_list_of_files(azure_connection.as_ref(), table_name, uploader)
                    .await;
            }
        }
    }

    pub async fn create_table_folder(&self, table_name: &str) {
        match self {
            Self::SqLite {
                repo: _,
                init_container: _,
            } => {}
            Self::AzureConnection(azure_connection) => {
                super::azure::create_table(azure_connection.as_ref(), table_name).await;
            }
        }
    }

    pub async fn save_table_file(
        &self,
        table_name: &str,
        table_file: &TableFile,
        content: Vec<u8>,
    ) {
        match self {
            Self::SqLite {
                repo,
                init_container: _,
            } => {
                repo.save_file(
                    table_name,
                    table_file.get_file_name().as_str(),
                    String::from_utf8(content).unwrap(),
                )
                .await;
                //super::sqlite::save_table_file(repo, table_name, table_file, content).await;
            }
            Self::AzureConnection(azure_connection) => {
                super::azure::save_table_file(
                    azure_connection.as_ref(),
                    table_name,
                    table_file.get_file_name().as_str(),
                    content,
                )
                .await;
            }
        }
        /*
           super::with_retries::save_table_file(
               self.azure_connection.as_ref(),
               table_name,
               table_file.get_file_name().as_str(),
               content,
           )
           .await;
        */
    }

    pub async fn delete_table_file(&self, table_name: &str, table_file: &TableFile) {
        match self {
            PersistIoOperations::AzureConnection(azure_connection) => {
                super::azure::delete_table_file(
                    azure_connection.as_ref(),
                    table_name,
                    table_file.get_file_name().as_str(),
                )
                .await;
            }
            PersistIoOperations::SqLite {
                repo,
                init_container: _,
            } => match table_file {
                TableFile::TableAttributes => {
                    repo.delete_file(table_name, TABLE_METADATA_FILE_NAME).await;
                }
                TableFile::DbPartition(partition_key) => {
                    repo.delete_file(table_name, partition_key.as_str()).await;
                }
            },
        }
    }

    pub async fn delete_table_folder(&self, table_name: &str) {
        match self {
            PersistIoOperations::AzureConnection(azure_connection) => {
                super::azure::delete_table_folder(azure_connection.as_ref(), table_name).await;
            }
            PersistIoOperations::SqLite {
                repo: _,
                init_container: _,
            } => {}
        }
    }

    pub async fn load_table_file(
        &self,
        table_name: &str,
        table_file: &TableFile,
    ) -> Option<Vec<u8>> {
        match self {
            PersistIoOperations::AzureConnection(azure_connection) => {
                super::azure::load_table_file(
                    azure_connection,
                    table_name,
                    table_file.get_file_name().as_str(),
                )
                .await
            }
            PersistIoOperations::SqLite {
                repo: _,
                init_container,
            } => {
                let mut read_access = init_container.lock().await;
                return read_access.get_file(table_name, table_file.get_file_name().as_str());
            }
        }
    }

    pub async fn init_table_from_other_source(
        &self,
        table_name: &str,
        db_table_snapshot: DbTableSnapshot,
    ) {
        match self {
            PersistIoOperations::AzureConnection(_) => {
                panic!("Files or Microsoft Azure Init is not supported");
            }
            PersistIoOperations::SqLite {
                repo,
                init_container: _,
            } => {
                super::sqlite::init_new_instance_table(repo, table_name, db_table_snapshot).await;
            }
        }
    }
}
