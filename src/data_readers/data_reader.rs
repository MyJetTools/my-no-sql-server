use std::collections::HashMap;

use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::RwLock,
};

use crate::utils::date_time::MyDateTime;

pub struct DataReadData {
    pub name: Option<String>,
    pub tcp_stream: Option<WriteHalf<TcpStream>>,
    pub ip: String,
    pub tables: HashMap<String, u8>,
    pub last_incoming_package: MyDateTime,
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
}

impl DataReader {
    pub fn new(id: u64, ip: String, tcp_stream: WriteHalf<TcpStream>) -> Self {
        let now = MyDateTime::utc_now();
        let data = DataReadData {
            name: None,
            tcp_stream: Some(tcp_stream),
            ip,
            tables: HashMap::new(),
            last_incoming_package: now,
        };

        Self {
            id,
            data: RwLock::new(data),
            connected: now,
        }
    }

    pub async fn to_string(&self) -> String {
        let data = self.data.read().await;
        return data.to_string();
    }

    pub async fn disconnect(&self) {
        let mut data = self.data.write().await;

        if data.tcp_stream.is_none() {
            println!("Socket {} is disconnected already", data.to_string());
            return;
        }

        let tcp_stream = data.tcp_stream.as_mut().unwrap();

        let result = tcp_stream.shutdown().await;

        if let Err(err) = result {
            println!("Can not shut down the socket: {:?}", err);
        }

        data.tcp_stream = None;
    }

    pub async fn send_package(&self, filter_by_table: Option<&str>, payload: &[u8]) {
        let mut data = self.data.write().await;

        if data.tcp_stream.is_none() {
            return;
        }

        if let Some(table_name) = filter_by_table {
            if !data.tables.contains_key(table_name) {
                return;
            }
        }

        let tcp_stream = data.tcp_stream.as_mut().unwrap();
        let result = tcp_stream.write_all(payload).await;

        if let Err(err) = result {
            println!(
                "Can not send data to the socket {}. Err: {:?}",
                data.to_string(),
                err
            );
        }
    }

    pub async fn set_socket_name(&self, set_socket_name: String) {
        let mut data = self.data.write().await;

        data.name = Some(set_socket_name);
    }

    pub async fn subscribe_to_table(&self, table_name: String) {
        let mut data = self.data.write().await;
        data.tables.insert(table_name, 0);
    }

    pub async fn update_last_incoming_moment(&self) {
        let now = MyDateTime::utc_now();
        let mut write_access = self.data.write().await;
        write_access.last_incoming_package.update(now);
    }
}
