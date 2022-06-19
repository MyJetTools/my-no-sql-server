use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::app::logs::Logs;
use tokio::sync::Mutex;

use super::{init_state_data::ProcessTableToLoad, InitStateData, InitStateSnapshot, TableToLoad};

pub struct InitState {
    data: Mutex<InitStateData>,
    tables_remains_to_init: AtomicUsize,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(InitStateData::new()),
            tables_remains_to_init: AtomicUsize::new(0),
        }
    }

    pub async fn init(&self, tables: Vec<String>, logs: &Logs) {
        self.tables_remains_to_init
            .store(tables.len(), Ordering::SeqCst);

        let mut write_access = self.data.lock().await;
        write_access.init_tables(tables, logs);
    }

    pub async fn get_next_table_to_init_files(&self) -> Option<Arc<TableToLoad>> {
        let mut write_access = self.data.lock().await;
        write_access.get_next_table_to_init_files()
    }

    pub async fn get_next_table_to_load(&self) -> ProcessTableToLoad {
        let mut write_access = self.data.lock().await;
        write_access.get_next_table_to_load()
    }

    pub async fn loaded_completed(&self, table_name: &str) {
        let mut write_access = self.data.lock().await;
        write_access.load_completed(table_name);
    }

    pub async fn get_snapshot(&self) -> InitStateSnapshot {
        let read_access = self.data.lock().await;
        read_access.get_snapshot()
    }

    pub async fn update_file_is_loaded(&self, table_name: &str) {
        let mut write_access = self.data.lock().await;
        write_access.update_file_is_loaded(table_name);
    }
}
