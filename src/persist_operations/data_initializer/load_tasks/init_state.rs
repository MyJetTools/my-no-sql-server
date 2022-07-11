use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    app::logs::Logs,
    db::{DbTableAttributesSnapshot, DbTableData},
    persist_io::TableListOfFilesUploader,
    persist_operations::data_initializer::LoadedTableItem,
};
use tokio::sync::Mutex;

use super::{init_state_data::NextFileToLoadResult, InitStateData, InitStateSnapshot};

pub struct InitState {
    data: Mutex<InitStateData>,

    tables_total: AtomicUsize,
    tables_loaded: AtomicUsize,

    files_total: AtomicUsize,
    files_loaded: AtomicUsize,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(InitStateData::new()),
            tables_total: AtomicUsize::new(0),
            files_total: AtomicUsize::new(0),
            files_loaded: AtomicUsize::new(0),
            tables_loaded: AtomicUsize::new(0),
        }
    }

    pub async fn init_table_names(&self, tables: Vec<String>, logs: &Logs) {
        println!("Added tables amount {}", tables.len());
        self.tables_total.store(tables.len(), Ordering::SeqCst);

        let mut write_access = self.data.lock().await;
        write_access.init_table_names(tables, logs);
    }

    pub async fn get_next_file_to_load(&self) -> NextFileToLoadResult {
        let mut write_acces = self.data.lock().await;
        write_acces.get_next_file_to_load()
    }

    pub async fn upload_table_file(
        &self,
        table_name: &str,
        file_name: String,
        table_item: LoadedTableItem,
    ) {
        self.files_loaded.fetch_add(1, Ordering::SeqCst);
        let mut write_access = self.data.lock().await;
        if write_access.upload_table_file_content(table_name, file_name, table_item) {
            self.tables_loaded.fetch_add(1, Ordering::SeqCst);
        }
    }

    pub async fn get_snapshot(&self) -> InitStateSnapshot {
        InitStateSnapshot {
            tables_total: self.tables_total.load(Ordering::SeqCst),
            tables_loaded: self.tables_loaded.load(Ordering::SeqCst),
            files_total: self.files_total.load(Ordering::SeqCst),
            files_loaded: self.files_loaded.load(Ordering::SeqCst),
        }
    }

    pub async fn get_table_data_result(&self) -> Option<(DbTableData, DbTableAttributesSnapshot)> {
        let mut write_access = self.data.lock().await;

        let (table_name, task) = write_access.remove_next_task()?;

        let (db_table_data, db_table_attributes) = task.get_result(table_name);

        Some((db_table_data, db_table_attributes))
    }
}

#[async_trait::async_trait]
impl TableListOfFilesUploader for InitState {
    async fn add_files(&self, table_name: &str, files: Vec<String>) {
        self.files_total.fetch_add(files.len(), Ordering::SeqCst);

        let mut write_access = self.data.lock().await;
        write_access.add_files_to_table(table_name, files);
    }

    async fn set_files_list_is_loaded(&self, table_name: &str) {
        let mut write_access = self.data.lock().await;
        write_access.set_file_list_is_loaded(table_name)
    }
}
