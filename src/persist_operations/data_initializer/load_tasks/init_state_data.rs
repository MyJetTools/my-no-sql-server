use std::collections::HashMap;

use crate::{
    app::logs::Logs, db_operations::validation,
    persist_operations::data_initializer::LoadedTableItem,
};

use super::LoadTableTask;

pub enum NextFileToLoadResult {
    FileToLoad {
        table_name: String,
        file_name: String,
    },
    NotReadyYet,
    NothingToLoad,
}

pub struct InitStateData {
    tables: HashMap<String, LoadTableTask>,
}

impl InitStateData {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn init_table_names(&mut self, tables_names: Vec<String>, logs: &Logs) {
        for table_name in tables_names {
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
                self.tables.insert(table_name, LoadTableTask::new());
            }
        }
    }

    pub fn get_next_file_to_load(&mut self) -> NextFileToLoadResult {
        let mut all_files_are_loaded_amount = 0;

        for (table_name, table_task) in &mut self.tables {
            if let Some(file_name) = table_task.get_next_file_to_load_content() {
                return NextFileToLoadResult::FileToLoad {
                    table_name: table_name.to_string(),
                    file_name,
                };
            }

            if table_task.is_file_list_loaded() {
                all_files_are_loaded_amount += 1;
            }
        }

        if all_files_are_loaded_amount == self.tables.len() {
            return NextFileToLoadResult::NothingToLoad;
        }

        return NextFileToLoadResult::NotReadyYet;
    }

    pub fn add_files_to_table(&mut self, table_name: &str, files: Vec<String>) {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.add_list_of_files(files);
        }
    }

    pub fn set_file_list_is_loaded(&mut self, table_name: &str) {
        if let Some(table) = self.tables.get_mut(table_name) {
            table.set_file_list_is_loaded();
        }
    }

    pub fn upload_table_file_content(
        &mut self,
        table_name: &str,
        file_name: String,
        table_item: LoadedTableItem,
    ) -> bool {
        if let Some(table) = self.tables.get_mut(table_name) {
            match table_item {
                LoadedTableItem::TableAttributes(attrs) => {
                    table.add_attribute(file_name, attrs);
                }
                LoadedTableItem::DbPartition {
                    partition_key,
                    db_partition,
                } => {
                    table.add_db_partition(file_name, partition_key, db_partition);
                }
            }

            return table.all_files_are_loaded();
        }

        return false;
    }

    fn get_first_table_name(&self) -> Option<String> {
        for key in self.tables.keys() {
            return Some(key.to_string());
        }
        None
    }

    pub fn remove_next_task(&mut self) -> Option<(String, LoadTableTask)> {
        let table_name = self.get_first_table_name()?;

        let result = self.tables.remove(&table_name)?;

        Some((table_name, result))
    }
}
