use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{date_time::MyDateTime, utils::ItemsOrNone};

#[derive(Debug, Clone, Copy)]
pub enum SystemProcess {
    System = 0,
    ServerSocket = 1,
    BlobOperation = 2,
    TableOperation = 3,
    Init = 4,
}

impl SystemProcess {
    pub fn parse(value: &str) -> Option<Self> {
        if value == "system" {
            return Some(SystemProcess::System);
        }

        if value == "serversocket" {
            return Some(SystemProcess::ServerSocket);
        }

        if value == "blob" {
            return Some(SystemProcess::BlobOperation);
        }

        if value == "table" {
            return Some(SystemProcess::TableOperation);
        }

        if value == "init" {
            return Some(SystemProcess::Init);
        }

        return None;
    }

    pub fn as_u8(&self) -> u8 {
        let result = *self as u8;
        return result;
    }
}

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

    pub process: SystemProcess,

    pub process_name: String,

    pub message: String,

    pub err_ctx: Option<String>,
}

struct LogsData {
    items: Vec<Arc<LogItem>>,
    items_by_table: HashMap<String, Vec<Arc<LogItem>>>,
    items_by_process: HashMap<u8, Vec<Arc<LogItem>>>,
}

pub struct Logs {
    data: RwLock<LogsData>,
}

impl Logs {
    pub fn new() -> Self {
        let logs_data = LogsData {
            items: Vec::new(),
            items_by_table: HashMap::new(),
            items_by_process: HashMap::new(),
        };

        Self {
            data: RwLock::new(logs_data),
        }
    }

    async fn add(&self, item: LogItem) {
        let item = Arc::new(item);
        let mut wirte_access = self.data.write().await;

        let process_id = item.as_ref().process.as_u8();

        add_table_data(
            &mut wirte_access.items_by_process,
            &process_id,
            item.clone(),
        );

        if let Some(table_name) = &item.table {
            add_table_data(&mut wirte_access.items_by_table, table_name, item.clone());
        }

        let items = &mut wirte_access.items;
        items.push(item);
        gc_logs(items);
    }

    pub async fn add_info(
        &self,
        table: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
    ) {
        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Info,
            table,
            process_name,
            process,
            message: message,
            err_ctx: None,
        };
        self.add(item).await;
    }

    pub async fn add_error(
        &self,
        table: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        err_ctx: Option<String>,
    ) {
        let item = LogItem {
            date: MyDateTime::utc_now(),
            level: LogLevel::Error,
            table,
            process_name,
            process,
            message: message,
            err_ctx,
        };

        self.add(item).await;
    }

    pub async fn handle_result(&self, log_item: Result<(), LogItem>) {
        if let Err(log_item) = log_item {
            self.add(log_item).await;
        }
    }

    pub async fn handle_aggregated_result(&self, items: Result<(), ItemsOrNone<LogItem>>) {
        if items.is_ok() {
            return;
        }

        let items = items.err().unwrap();

        let items = items.consume();

        if items.is_none() {
            return;
        }

        let mut items = items.unwrap();

        for item in items.drain(..) {
            self.add(item).await;
        }
    }

    pub async fn get(&self) -> Vec<Arc<LogItem>> {
        let read_access = self.data.read().await;
        read_access.items.to_vec()
    }

    pub async fn get_by_table_name(&self, table_name: &str) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_table.get(table_name)?;
        return Some(result.to_vec());
    }

    pub async fn get_by_process(&self, process: SystemProcess) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_process.get(&process.as_u8())?;
        return Some(result.to_vec());
    }
}

fn add_table_data<T>(
    items_by_table: &mut HashMap<T, Vec<Arc<LogItem>>>,
    category: &T,
    item: Arc<LogItem>,
) where
    T: Eq + std::hash::Hash + Clone + Sized,
{
    if !items_by_table.contains_key(category) {
        items_by_table.insert(category.clone(), Vec::new());
    }

    let items = items_by_table.get_mut(category).unwrap();

    items.push(item);

    gc_logs(items);
}

fn gc_logs(items: &mut Vec<Arc<LogItem>>) {
    while items.len() > 100 {
        items.remove(0);
    }
}
