use std::collections::HashMap;

use my_no_sql_core::db::{DbPartition, DbTableAttributesSnapshot, DbTableInner};
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum FileStatus {
    Waiting,
    Loading,
    DbPartition {
        partition_key: String,
        db_partition: DbPartition,
    },
}

impl FileStatus {
    pub fn is_waiting(&self) -> bool {
        match self {
            FileStatus::Waiting => true,
            _ => false,
        }
    }
}

pub struct LoadTableTask {
    files_list_is_loaded: bool,
    files: HashMap<String, FileStatus>,
    attr: Option<DbTableAttributesSnapshot>,
}

impl LoadTableTask {
    pub fn new() -> Self {
        Self {
            files_list_is_loaded: false,
            files: HashMap::new(),
            attr: None,
        }
    }

    pub fn add_list_of_files(&mut self, files: Vec<String>) {
        for file in files {
            self.files.insert(file, FileStatus::Waiting);
        }
    }

    pub fn get_next_file_to_load_content(&mut self) -> Option<String> {
        let next_file = self.get_next_file_name_to_load_content()?;

        self.files
            .insert(next_file.to_string(), FileStatus::Loading);

        Some(next_file)
    }

    fn get_next_file_name_to_load_content(&self) -> Option<String> {
        for (file_name, status) in &self.files {
            if status.is_waiting() {
                return Some(file_name.clone());
            }
        }

        None
    }

    pub fn add_db_partition(
        &mut self,
        file_name: String,
        partition_key: String,
        db_partition: DbPartition,
    ) {
        self.files.insert(
            file_name,
            FileStatus::DbPartition {
                partition_key,
                db_partition,
            },
        );
    }

    pub fn add_attribute(&mut self, file_name: String, attr: DbTableAttributesSnapshot) {
        self.attr = Some(attr);
        self.files.remove(file_name.as_str());
    }

    pub fn is_file_list_loaded(&self) -> bool {
        self.files_list_is_loaded
    }

    pub fn set_file_list_is_loaded(&mut self) {
        self.files_list_is_loaded = true;
    }

    pub fn get_result(mut self, table_name: String) -> (DbTableInner, DbTableAttributesSnapshot) {
        let mut db_table_data = DbTableInner::new(table_name, DateTimeAsMicroseconds::now());

        let mut attr = self.attr.take();

        if attr.is_none() {
            attr = Some(DbTableAttributesSnapshot::create_default())
        }

        for (_, file_status) in self.files {
            match file_status {
                FileStatus::Waiting => {
                    panic!("Somehow we started getting result having Waiting File")
                }
                FileStatus::Loading => {
                    panic!("Somehow we started getting result having Loading File")
                }
                FileStatus::DbPartition {
                    partition_key,
                    db_partition,
                } => {
                    db_table_data.partitions.insert(partition_key, db_partition);
                }
            }
        }

        (db_table_data, attr.unwrap())
    }

    pub fn all_files_are_loaded(&self) -> bool {
        for file in self.files.values() {
            match file {
                FileStatus::Waiting => return false,
                FileStatus::Loading => {
                    return false;
                }
                FileStatus::DbPartition {
                    partition_key: _,
                    db_partition: _,
                } => {}
            }
        }

        true
    }
}
