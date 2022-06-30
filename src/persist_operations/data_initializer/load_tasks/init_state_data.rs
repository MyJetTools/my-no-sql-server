use std::{collections::HashMap, sync::Arc};

use crate::{
    app::logs::Logs, db_operations::validation,
    persist_operations::data_initializer::LoadedTableItem,
};

use super::{TableFilesToLoad, TableLoadingTask, TableToLoadListOfFiles};

pub enum NextFileToLoadResult {
    DataToLoad {
        table_loading_task: Arc<TableLoadingTask>,
        file_name: String,
    },
    NotReadyYeat,
    NothingToLoad,
}

pub struct InitStateData {
    pub tables_to_load_list_of_files: TableToLoadListOfFiles,
    pub tables_files_to_load: HashMap<String, TableFilesToLoad>,
    pub tables_loading_tasks: HashMap<String, Arc<TableLoadingTask>>,
}

impl InitStateData {
    pub fn new() -> Self {
        Self {
            tables_to_load_list_of_files: TableToLoadListOfFiles::new(),
            tables_files_to_load: HashMap::new(),
            tables_loading_tasks: HashMap::new(),
        }
    }

    pub fn get_next_table_to_load_list_of_files(&mut self) -> Option<String> {
        self.tables_to_load_list_of_files.get_next()
    }

    pub fn init_table_names(&mut self, tables: Vec<String>, logs: &Logs) {
        for table_name in tables {
            if let Err(err) = validation::validate_table_name(table_name.as_str()) {
                logs.add_error(
                    Some(table_name),
                    crate::app::logs::SystemProcess::Init,
                    "init_tables".to_string(),
                    format!(
                        "Table name does not fit validation. Skipping loading it... Reason:{:?}",
                        err
                    ),
                    None,
                );
            } else {
                self.tables_to_load_list_of_files
                    .add_table(table_name.to_string());

                self.tables_files_to_load.insert(
                    table_name.to_string(),
                    TableFilesToLoad::new(table_name.to_string()),
                );

                self.tables_loading_tasks.insert(
                    table_name.to_string(),
                    Arc::new(TableLoadingTask::new(table_name)),
                );
            }
        }
    }

    pub fn get_next_file_to_load(&mut self) -> NextFileToLoadResult {
        if self.tables_files_to_load.len() == 0 {
            if self.tables_to_load_list_of_files.has_something_to_process() {
                return NextFileToLoadResult::NotReadyYeat;
            }
        }

        for files_to_load in self.tables_files_to_load.values_mut() {
            if let Some(file_name) = files_to_load.get_next_file_to_load() {
                if let Some(loading_task) = self
                    .tables_loading_tasks
                    .get(files_to_load.table_name.as_str())
                {
                    return NextFileToLoadResult::DataToLoad {
                        table_loading_task: loading_task.clone(),
                        file_name,
                    };
                }
            }
        }

        NextFileToLoadResult::NothingToLoad
    }

    pub fn add_files_to_table(&mut self, table_name: &str, files: Vec<String>) {
        if let Some(tables_loading_tasks) = self.tables_loading_tasks.get_mut(table_name) {
            tables_loading_tasks.add_total_files_amount(files.len());
        }

        if let Some(table_files_to_load) = self.tables_files_to_load.get_mut(table_name) {
            table_files_to_load.add_files(files);
        }
    }

    pub fn set_file_list_is_loaded(&mut self, table_name: &str) {
        if let Some(tables_loading_tasks) = self.tables_loading_tasks.get_mut(table_name) {
            tables_loading_tasks.set_file_list_is_loaded();
        }

        if let Some(table_files_to_load) = self.tables_files_to_load.get_mut(table_name) {
            table_files_to_load.file_list_is_loaded();
        }
    }

    fn get_first_table_task_key(&self) -> Option<String> {
        for table_name in self.tables_loading_tasks.keys() {
            return Some(table_name.to_string());
        }

        None
    }

    pub fn get_loading_task_as_result(&mut self) -> Option<Arc<TableLoadingTask>> {
        let table_name = self.get_first_table_task_key()?;
        let result = self.tables_loading_tasks.remove(&table_name)?;
        return Some(result);
    }

    pub async fn upload_table_file(&self, table_name: &str, table_item: LoadedTableItem) -> bool {
        if let Some(table_task) = self.tables_loading_tasks.get(table_name) {
            return table_task.add_loaded_file(table_item).await;
        }

        false
    }
}
