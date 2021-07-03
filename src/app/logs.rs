use tokio::sync::RwLock;

use crate::utils::date_time::MyDateTime;

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

pub struct Logs {
    items: RwLock<Vec<LogItem>>,
}

impl Logs {
    pub fn new() -> Self {
        Self {
            items: RwLock::new(Vec::new()),
        }
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

        let mut wirte_access = self.items.write().await;

        wirte_access.push(item);

        while wirte_access.len() > 100 {
            wirte_access.remove(0);
        }
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

        let mut wirte_access = self.items.write().await;

        wirte_access.push(item);

        while wirte_access.len() > 100 {
            wirte_access.remove(0);
        }
    }

    pub async fn get(&self) -> Vec<LogItem> {
        let read_access = self.items.read().await;
        read_access.to_vec()
    }
}
