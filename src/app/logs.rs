use std::{collections::HashMap, sync::Arc};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy)]
pub enum SystemProcess {
    System = 0,
    TcpSocket = 1,
    BlobOperation = 2,
    TableOperation = 3,
    Init = 4,
}

impl SystemProcess {
    pub fn iterate() -> Vec<Self> {
        let mut result = Vec::new();

        result.push(SystemProcess::System);
        result.push(SystemProcess::TcpSocket);
        result.push(SystemProcess::BlobOperation);
        result.push(SystemProcess::TableOperation);
        result.push(SystemProcess::Init);

        return result;
    }
    pub fn parse(value: &str) -> Option<Self> {
        if value == "system" {
            return Some(SystemProcess::System);
        }

        if value == "tcpsocket" {
            return Some(SystemProcess::TcpSocket);
        }

        if value == "bloboperation" {
            return Some(SystemProcess::BlobOperation);
        }

        if value == "tableoperation" {
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
    FatalError,
}
#[derive(Debug, Clone)]
pub struct LogItem {
    pub date: DateTimeAsMicroseconds,

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

        match &item.level {
            LogLevel::Info => {}
            LogLevel::Error => print_to_console(&item),
            LogLevel::FatalError => print_to_console(&item),
        }

        match &item.process {
            SystemProcess::System => print_to_console(&item),
            SystemProcess::TcpSocket => {}
            SystemProcess::BlobOperation => {}
            SystemProcess::TableOperation => {}
            SystemProcess::Init => print_to_console(&item),
        }

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
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::Info,
            table,
            process_name,
            process,
            message: message,
            err_ctx: None,
        };

        if let SystemProcess::BlobOperation = process {
            print_to_console(&item);
        }

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
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::Error,
            table,
            process_name,
            process,
            message: message,
            err_ctx,
        };

        self.add(item).await;
    }

    pub async fn add_fatal_error(
        &self,
        process: SystemProcess,
        process_name: String,
        message: String,
    ) {
        let item = LogItem {
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::FatalError,
            table: None,
            process_name,
            process,
            message: message,
            err_ctx: None,
        };

        self.add(item).await;
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

fn print_to_console(item: &LogItem) {
    println!(
        "{} {:?} {:?} -----------",
        item.date.to_rfc3339(),
        item.level,
        item.process
    );
    if let Some(table) = &item.table {
        println!("Table: {}", table);
    }
    println!("Process: {}", item.process_name);
    println!("Message: {}", item.message);
    if let Some(err_ctx) = &item.err_ctx {
        println!("Err_ctx: {}", err_ctx);
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
