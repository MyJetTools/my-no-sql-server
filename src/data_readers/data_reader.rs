use std::collections::HashMap;

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::RwLock,
};

use crate::{
    app::logs::LogItem,
    date_time::{AtomicDateTime, MyDateTime},
};

pub struct DataReadData {
    pub name: Option<String>,
    pub tcp_stream: Option<WriteHalf<TcpStream>>,
    pub ip: String,
    pub tables: HashMap<String, u8>,
}

impl DataReadData {
    pub fn to_string(&self) -> String {
        match &self.name {
            Some(name) => return format!("{} {}", name, self.ip),
            None => self.ip.clone(),
        }
    }
}

pub struct DataReader {
    pub data: RwLock<DataReadData>,
    pub id: u64,
    pub connected: MyDateTime,
    pub last_incoming_package: AtomicDateTime,
}

impl DataReader {
    pub fn new(id: u64, ip: String, tcp_stream: WriteHalf<TcpStream>) -> Self {
        let now = MyDateTime::utc_now();
        let data = DataReadData {
            name: None,
            tcp_stream: Some(tcp_stream),
            ip,
            tables: HashMap::new(),
        };

        Self {
            id,
            data: RwLock::new(data),
            connected: now,
            last_incoming_package: AtomicDateTime::from_date_time(now),
        }
    }

    pub async fn to_string(&self) -> String {
        let data = self.data.read().await;
        return data.to_string();
    }

    fn get_process_name(&self, sub_process: &str) -> String {
        format!("DataReader[{}]::{}", self.id, sub_process)
    }

    pub async fn disconnect(&self) -> Result<(), LogItem> {
        let mut data = self.data.write().await;

        if data.tcp_stream.is_none() {
            let err = LogItem {
                date: MyDateTime::utc_now(),
                table: None,
                level: crate::app::logs::LogLevel::Info,
                process: crate::app::logs::SystemProcess::ServerSocket,
                err_ctx: None,
                message: format!("Socket {} is disconnected already", data.to_string()),
                process_name: self.get_process_name("disconnect"),
            };

            return Err(err);
        }

        let tcp_stream = data.tcp_stream.as_mut().unwrap();

        let result = tcp_stream.shutdown().await;

        if let Err(err) = result {
            let err = LogItem {
                date: MyDateTime::utc_now(),
                table: None,
                level: crate::app::logs::LogLevel::Info,
                process: crate::app::logs::SystemProcess::ServerSocket,
                err_ctx: None,
                message: format!("Can not shut down the socket: {:?}", err),
                process_name: self.get_process_name("disconnect"),
            };

            return Err(err);
        }

        data.tcp_stream = None;

        Ok(())
    }

    pub async fn send_package(
        &self,
        filter_by_table: Option<&str>,
        payload: &[u8],
    ) -> Result<(), LogItem> {
        let mut data = self.data.write().await;

        if data.tcp_stream.is_none() {
            return Ok(());
        }

        if let Some(table_name) = filter_by_table {
            if !data.tables.contains_key(table_name) {
                return Ok(());
            }
        }

        let tcp_stream = data.tcp_stream.as_mut().unwrap();
        let result = tcp_stream.write_all(payload).await;

        if let Err(err) = result {
            let err = LogItem {
                table: crate::utils::options_utils::clone_string_value(filter_by_table),
                date: MyDateTime::utc_now(),
                level: crate::app::logs::LogLevel::Info,
                process: crate::app::logs::SystemProcess::ServerSocket,
                message: format!(
                    "Can not send data to the socket {}. Err: {:?}",
                    data.to_string(),
                    err
                ),
                err_ctx: None,
                process_name: "send_package".to_string(),
            };

            return Err(err);
        }

        Ok(())
    }

    pub async fn set_socket_name(&self, set_socket_name: String) {
        let mut data = self.data.write().await;

        data.name = Some(set_socket_name);
    }

    pub async fn subscribe_to_table(&self, table_name: String) {
        let mut data = self.data.write().await;
        data.tables.insert(table_name, 0);
    }
}
