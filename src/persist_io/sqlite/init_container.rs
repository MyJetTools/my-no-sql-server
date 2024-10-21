use std::collections::BTreeMap;

use crate::{persist_io::TABLE_METADATA_FILE_NAME, sqlite_repo::MyNoSqlFileDto};

#[derive(Default)]
pub struct InitTableFiles {
    files: BTreeMap<String, Vec<u8>>,
}

impl InitTableFiles {
    pub fn can_be_gc(&self) -> bool {
        self.files.is_empty()
    }
}

pub struct InitContainer {
    by_table: BTreeMap<String, InitTableFiles>,
}

impl InitContainer {
    pub fn new() -> Self {
        Self {
            by_table: BTreeMap::new(),
        }
    }

    pub fn init(&mut self, dto_rows: Vec<MyNoSqlFileDto>) {
        for dto in dto_rows {
            if let Some(table) = self.by_table.get_mut(&dto.table_name) {
                table.files.insert(dto.file_name, dto.content.into_bytes());
            } else {
                let mut table_files = InitTableFiles::default();
                table_files
                    .files
                    .insert(dto.file_name, dto.content.into_bytes());
                self.by_table.insert(dto.table_name, table_files);
            }
        }
    }

    pub fn get_file_names(&self, table_name: &str) -> Vec<String> {
        let mut result = Vec::new();

        result.push(TABLE_METADATA_FILE_NAME.to_string());

        if let Some(table_data) = self.by_table.get(table_name) {
            for pk in table_data.files.keys() {
                result.push(pk.to_string());
            }
        }

        result
    }

    pub fn get_list_of_tables(&self) -> Vec<String> {
        self.by_table.keys().map(|itm| itm.to_string()).collect()
    }

    pub fn get_file(&mut self, table_name: &str, partition_key: &str) -> Option<Vec<u8>> {
        let table_data = self.by_table.get_mut(table_name)?;

        let result = table_data.files.remove(partition_key);

        if table_data.can_be_gc() {
            self.by_table.remove(table_name);
        }

        result
    }

    pub fn get_files_by_table(&mut self, table_name: &str) -> Option<BTreeMap<String, Vec<u8>>> {
        self.by_table.remove(table_name).map(|itm| itm.files)
    }
}
