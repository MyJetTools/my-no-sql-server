use std::sync::Arc;

use crate::{
    db::{db_snapshots::DbPartitionSnapshot, DbTableAttributesSnapshot},
    persist_io::{
        active_loader::{ActiveLoaderState, ActiveLoaders},
        serializers, PersistIoOperations, TableLoadItem,
    },
};

use super::loader::TableLoaderFromFile;

pub struct FilesPersistIo {
    path: String,
    path_separator: char,
    loaders: ActiveLoaders<TableLoaderFromFile>,
}

impl FilesPersistIo {
    pub fn new(path: String, path_separator: char) -> Self {
        let mut path = if path.starts_with("~") {
            format!("{}{}", env!("HOME"), &path[1..])
        } else {
            path.to_string()
        };

        if !path.ends_with(path_separator) {
            path.push(path_separator)
        }

        Self {
            path,
            path_separator,
            loaders: ActiveLoaders::new(),
        }
    }

    fn compile_table_dir_path(&self, table_name: &str) -> String {
        format!("{path}{table_name}", path = self.path,)
    }

    fn compile_partition_file_name(&self, table_name: &str, partition_key: &str) -> String {
        format!(
            "{path}{path_separator}{table_name}{path_separator}{partition_key}",
            path = self.path,
            path_separator = self.path_separator,
            partition_key = super::super::serializers::blob_file_name::encode(partition_key),
        )
    }

    fn compile_attrs_file_name(&self, table_name: &str) -> String {
        format!(
            "{path}{path_separator}{table_name}{path_separator}{attrs_file_name}",
            path = self.path,
            path_separator = self.path_separator,
            attrs_file_name = serializers::table_attrs::METADATA_FILE_NAME,
        )
    }
}

#[async_trait::async_trait]
impl PersistIoOperations for FilesPersistIo {
    async fn get_list_of_tables(&self) -> Vec<String> {
        let mut result = Vec::new();

        let path = self.path.as_str();

        for entry in std::fs::read_dir(path).unwrap() {
            if let Ok(entity) = entry {
                let file_type = entity.file_type().unwrap();

                if file_type.is_dir() {
                    let path = entity.path();

                    let path = format!("{}", path.display());

                    result.push(
                        super::loader::extract_file_name(path.as_str(), self.path_separator)
                            .to_string(),
                    );
                }
            }
        }

        result
    }

    async fn start_loading_table(&self, table_name: &str) -> Option<TableLoadItem> {
        match self.loaders.get(table_name).await {
            Some(_) => {
                panic!("You can not start loading table {} twice", table_name);
            }
            None => {
                let loader = TableLoaderFromFile::new(
                    self.compile_table_dir_path(table_name),
                    self.path_separator,
                )
                .await;

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
        let table_dir_path = self.compile_table_dir_path(table_name);
        tokio::fs::create_dir(table_dir_path.as_str())
            .await
            .unwrap();

        self.save_table_attributes(table_name, attr).await;
    }

    async fn save_table_attributes(&self, table_name: &str, attr: &DbTableAttributesSnapshot) {
        let payload = serializers::table_attrs::serialize(attr);
        let attr_full_path = self.compile_attrs_file_name(table_name);
        tokio::fs::write(attr_full_path, payload).await.unwrap();
    }

    async fn save_partition(
        &self,
        table_name: &str,
        partition_key: &str,
        db_partition_snapshot: &DbPartitionSnapshot,
    ) {
        let partition_file_name = self.compile_partition_file_name(table_name, partition_key);

        let payload = db_partition_snapshot
            .db_rows_snapshot
            .as_json_array()
            .build();

        tokio::fs::write(partition_file_name, payload)
            .await
            .unwrap();
    }

    async fn delete_table(&self, table_name: &str) {
        let table_dir_path = self.compile_table_dir_path(table_name);
        tokio::fs::remove_dir_all(table_dir_path).await.unwrap();
    }

    async fn delete_partition(&self, table_name: &str, partition_key: &str) {
        let partition_file_name = self.compile_partition_file_name(table_name, partition_key);
        tokio::fs::remove_file(partition_file_name).await.unwrap();
    }
}
