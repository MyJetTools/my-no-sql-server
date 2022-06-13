use std::sync::Arc;

use crate::{app::logs::Logs, db_operations::validation};

use super::{InitStateSnapshot, InitTableStateSnapshot, TableToLoad};

pub struct InitStateData {
    pub tables_to_load: Vec<Arc<TableToLoad>>,
    pub tables_being_loaded: Vec<Arc<TableToLoad>>,
    pub tables_loaded: Vec<Arc<TableToLoad>>,
}

impl InitStateData {
    pub fn new() -> Self {
        Self {
            tables_to_load: Vec::new(),
            tables_being_loaded: Vec::new(),
            tables_loaded: Vec::new(),
        }
    }

    pub fn init_tables(&mut self, tables: Vec<String>, logs: &Logs) -> Vec<Arc<TableToLoad>> {
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
                self.tables_to_load
                    .push(Arc::new(TableToLoad::new(table_name)));
            }
        }
        return self.tables_to_load.clone();
    }

    pub fn get_next_table_to_load(&mut self) -> Option<Arc<TableToLoad>> {
        let next_table_to_load = {
            if self.tables_to_load.len() > 0 {
                Some(self.tables_to_load.remove(0))
            } else {
                None
            }
        }?;

        self.tables_being_loaded.push(next_table_to_load.clone());

        Some(next_table_to_load)
    }

    pub fn load_completed(&mut self, table_name: &str) {
        let index = self
            .tables_being_loaded
            .iter()
            .position(|item| item.table_name == table_name);

        if index.is_none() {
            panic!(
                "Somehow we did not found table {} as being loaded",
                table_name
            );
        }

        let index = index.unwrap();

        let removed = self.tables_being_loaded.remove(index);
        self.tables_loaded.push(removed);
    }

    pub fn get_snapshot(&self) -> InitStateSnapshot {
        InitStateSnapshot {
            to_load: convert_to_tables_snapshot(&self.tables_to_load),
            loading: convert_to_tables_snapshot(&self.tables_being_loaded),
            loaded: convert_to_tables_snapshot(&self.tables_loaded),
        }
    }

    pub fn update_file_is_loaded(&mut self, table_name: &str) {
        for table in &self.tables_being_loaded {
            if table.table_name == table_name {
                table.inc_files_loaded();
                return;
            }
        }
    }
}

fn convert_to_tables_snapshot(src: &Vec<Arc<TableToLoad>>) -> Vec<InitTableStateSnapshot> {
    if src.len() == 0 {
        return Vec::new();
    }

    let mut result = Vec::with_capacity(src.len());

    for table_to_load in src.iter() {
        result.push(convert_to_table_snapshot(table_to_load));
    }

    result
}

fn convert_to_table_snapshot(src: &Arc<TableToLoad>) -> InitTableStateSnapshot {
    InitTableStateSnapshot {
        name: src.table_name.clone(),
        to_load: src.get_files_to_load(),
        loaded: src.get_files_loaded(),
        list_is_loaded: src.get_files_list_is_loaded(),
        init_started: src.get_initializing_is_started(),
    }
}
