use std::collections::HashMap;

use crate::{app::logs::Logs, db_operations::validation};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;
#[derive(Debug, Clone)]
pub struct TableInitState {
    pub partitions_amount: usize,
    pub loaded: usize,
    pub started: DateTimeAsMicroseconds,
}

pub struct InitState {
    data: Mutex<HashMap<String, TableInitState>>,
}

impl InitState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub async fn init(&self, tables: &[String], logs: &Logs) {
        let mut write_access = self.data.lock().await;
        for table_name in tables {
            if let Err(err) = validation::validate_table_name(table_name) {
                logs.add_error(
                    Some(table_name.to_string()),
                    crate::app::logs::SystemProcess::Init,
                    "init_tables".to_string(),
                    format!(
                        "Table name does not fit validation. Skipping loading it... Reason:{:?}",
                        err
                    ),
                    None,
                );
            } else {
                write_access.insert(
                    table_name.to_string(),
                    TableInitState {
                        partitions_amount: 0,
                        loaded: 0,
                        started: DateTimeAsMicroseconds::now(),
                    },
                );
            }
        }
    }

    pub async fn update_partitions_amount(&self, table_name: &str, amount: usize) {
        let mut write_access = self.data.lock().await;
        if let Some(item) = write_access.get_mut(table_name) {
            item.partitions_amount = amount;
            item.started = DateTimeAsMicroseconds::now();
        }
    }

    pub async fn update_loaded(&self, table_name: &str, amount: usize) {
        let mut write_access = self.data.lock().await;
        if let Some(item) = write_access.get_mut(table_name) {
            item.loaded = amount;
        }
    }

    pub async fn loaded_completed(&self, table_name: &str) {
        let mut write_access = self.data.lock().await;
        write_access.remove(table_name);
    }

    pub async fn get_snapshot(&self) -> (HashMap<String, TableInitState>, usize) {
        let mut result = HashMap::new();
        let read_access = self.data.lock().await;

        for (table_name, item) in &*read_access {
            if item.partitions_amount > 0 || item.loaded > 0 {
                result.insert(table_name.to_string(), item.clone());
            }
        }

        (result, read_access.len())
    }
}
