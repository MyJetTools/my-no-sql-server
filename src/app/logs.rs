use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::date_time::MyDateTime;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
}
#[derive(Debug, Clone)]
pub struct LogItem {
    pub date: MyDateTime,

    pub table: Option<String>,

    pub level: LogLevel,

    pub process: String,

    pub message: String,

    pub err_ctx: Option<String>,
}

struct LogsData {
    items: Vec<LogItem>,
    items_by_table: HashMap<String, Vec<LogItem>>,
}

pub struct Logs {
    data: RwLock<LogsData>,
}

impl Logs {
    pub fn new() -> Self {
        let logs_data = LogsData {
            items: Vec::new(),
            items_by_table: HashMap::new(),
        };

        Self {
            data: RwLock::new(logs_data),
        }
    }

    async fn add(&self, item: LogItem) {
        let mut wirte_access = self.data.write().await;

        if let Some(table_name) = &item.table {
            if !wirte_access.items_by_table.contains_key(table_name) {
                wirte_access
                    .items_by_table
                    .insert(table_name.to_string(), Vec::new());
            }

            let items = wirte_access.items_by_table.get_mut(table_name).unwrap();

            items.push(item.clone());

            gc_logs(items);
        }

        let items = &mut wirte_access.items;
        items.push(item);
        gc_logs(items);
    }

    pub async fn add_info(&self, table: Option<String>, process: String, message: String) {
        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Info,
            table,
            process: process,
            message: message,
            err_ctx: None,
        };
        self.add(item).await;
    }

    pub async fn add_error(
        &self,
        table: Option<String>,
        process: String,
        message: String,
        err_ctx: Option<String>,
    ) {
        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Error,
            table,
            process: process,
            message: message,
            err_ctx,
        };

        self.add(item).await;
    }

    pub async fn get(&self) -> Vec<LogItem> {
        let read_access = self.data.read().await;
        read_access.items.to_vec()
    }
}

fn gc_logs(items: &mut Vec<LogItem>) {
    while items.len() > 100 {
        items.remove(0);
    }
}
