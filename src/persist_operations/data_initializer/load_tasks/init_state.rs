use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    persist_io::TableListOfFilesUploader, persist_operations::data_initializer::LoadedTableItem,
};
use my_no_sql_sdk::core::db::DbTable;
use tokio::sync::Mutex;

use super::{init_state_data::NextFileToLoadResult, InitStateData, InitStateSnapshot};

pub struct InitState {
    data: Mutex<Option<InitStateData>>,

    tables_total: AtomicUsize,
    tables_loaded: AtomicUsize,

    files_total: AtomicUsize,
    files_loaded: AtomicUsize,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(Some(InitStateData::new())),
            tables_total: AtomicUsize::new(0),
            files_total: AtomicUsize::new(0),
            files_loaded: AtomicUsize::new(0),
            tables_loaded: AtomicUsize::new(0),
        }
    }

    fn get_init_state_mut(init_state_data: &mut Option<InitStateData>) -> &mut InitStateData {
        match init_state_data.as_mut() {
            Some(init_state_data) => init_state_data,
            None => panic!("Init State Data is disposed"),
        }
    }

    pub async fn init_table_names(&self, tables: Vec<String>) {
        println!("Added tables amount {}", tables.len());
        self.tables_total.store(tables.len(), Ordering::SeqCst);

        let mut write_access = self.data.lock().await;

        Self::get_init_state_mut(&mut write_access).init_table_names(tables);
    }

    pub async fn get_next_file_to_load(&self) -> NextFileToLoadResult {
        let mut write_access = self.data.lock().await;
        Self::get_init_state_mut(&mut write_access).get_next_file_to_load()
    }

    pub async fn upload_table_file(
        &self,
        table_name: &str,
        file_name: String,
        table_item: LoadedTableItem,
    ) {
        self.files_loaded.fetch_add(1, Ordering::SeqCst);
        let mut write_access = self.data.lock().await;
        let write_access = Self::get_init_state_mut(&mut write_access);
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

    pub async fn get_table_data_result(&self) -> Option<DbTable> {
        let mut write_access = self.data.lock().await;
        let write_access = Self::get_init_state_mut(&mut write_access);

        let (table_name, task) = write_access.remove_next_task()?;

        let db_table = task.get_result(table_name);

        Some(db_table)
    }

    pub async fn dispose(&self) {
        let mut write_access = self.data.lock().await;
        *write_access = None;
    }
}

#[async_trait::async_trait]
impl TableListOfFilesUploader for InitState {
    async fn add_files(&self, table_name: &str, files: Vec<String>) {
        self.files_total.fetch_add(files.len(), Ordering::SeqCst);

        let mut write_access = self.data.lock().await;
        write_access
            .as_mut()
            .unwrap()
            .add_files_to_table(table_name, files);
    }

    async fn set_files_list_is_loaded(&self, table_name: &str) {
        let mut write_access = self.data.lock().await;
        write_access
            .as_mut()
            .unwrap()
            .set_file_list_is_loaded(table_name)
    }
}
