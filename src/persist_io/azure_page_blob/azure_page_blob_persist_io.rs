use std::sync::Arc;

use my_azure_storage_sdk::AzureStorageConnection;

use crate::{
    app::logs::Logs,
    db::{db_snapshots::DbPartitionSnapshot, DbTableAttributesSnapshot},
    persist_io::{
        active_loader::{ActiveLoaderState, ActiveLoaders},
        PersistIoOperations, TableLoadItem,
    },
};

use super::table::PageBlobTableLoader;

pub struct AzurePageBlobPersistIo {
    logs: Arc<Logs>,
    azure_connection: Arc<AzureStorageConnection>,
    loaders: ActiveLoaders<PageBlobTableLoader>,
}

impl AzurePageBlobPersistIo {
    pub fn new(azure_connection: Arc<AzureStorageConnection>, logs: Arc<Logs>) -> Self {
        Self {
            logs,
            azure_connection,
            loaders: ActiveLoaders::new(),
        }
    }
}

#[async_trait::async_trait]
impl PersistIoOperations for AzurePageBlobPersistIo {
    async fn get_list_of_tables(&self) -> Vec<String> {
        super::table::get_list(&self.azure_connection)
            .await
            .unwrap()
    }

    async fn start_loading_table(&self, table_name: &str) -> Option<TableLoadItem> {
        match self.loaders.get(table_name).await {
            Some(_) => {
                panic!("You can not start loading table {} twice", table_name);
            }
            None => {
                let loader =
                    PageBlobTableLoader::new(self.azure_connection.clone(), table_name).await;

                let result = loader.get_next().await;

                if result.is_none() {
                    self.loaders.set_as_finished(table_name).await;
                } else {
                    self.loaders.set_loader(table_name, Arc::new(loader)).await;
                }

                return result;
            }
        }
    }

    async fn continue_loading_table(&self, table_name: &str) -> Option<TableLoadItem> {
        match self.loaders.get(table_name).await {
            Some(loader) => {
                match loader {
                    ActiveLoaderState::Active(loader) => {
                        let result = loader.get_next().await;

                        if result.is_none() {
                            self.loaders.set_as_finished(table_name).await;
                        }

                        return result;
                    }
                    ActiveLoaderState::Finished => {
                        panic!("Can not load next table item with not finished status");
                    }
                };
            }
            None => {
                panic!("First loading table {} should be initialized", table_name);
            }
        }
    }

    async fn create_table(&self, table_name: &str, attr: &DbTableAttributesSnapshot) {
        super::create_table::with_retries(
            self.logs.as_ref(),
            &self.azure_connection,
            table_name,
            attr,
        )
        .await;
    }

    async fn save_table_attributes(&self, table_name: &str, attr: &DbTableAttributesSnapshot) {
        super::save_table_attributes::with_retries(
            self.logs.as_ref(),
            &self.azure_connection,
            table_name,
            attr,
        )
        .await;
    }

    async fn save_partition(
        &self,
        table_name: &str,
        partition_key: &str,
        db_partition_snapshot: &DbPartitionSnapshot,
    ) {
        super::save_partition::with_retries(
            self.logs.as_ref(),
            &self.azure_connection,
            table_name,
            partition_key,
            db_partition_snapshot,
        )
        .await;
    }

    async fn delete_table(&self, table_name: &str) {
        super::delete_table::with_retries(
            self.logs.clone(),
            self.azure_connection.clone(),
            table_name,
        )
        .await;
    }

    async fn delete_partition(&self, table_name: &str, partition_key: &str) {
        super::delete_partition::with_retries(
            self.logs.as_ref(),
            self.azure_connection.as_ref(),
            table_name,
            partition_key,
        )
        .await;
    }
}
