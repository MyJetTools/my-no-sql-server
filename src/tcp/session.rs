use std::{collections::HashMap, sync::Arc};

use my_no_sql_tcp_shared::TcpContract;
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};
use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

use crate::app::{self, logs::Logs};

use super::SessionMetrics;

#[derive(Debug)]
pub enum SendPackageError {
    JustDisconnected,
    Disconnected,
}

pub struct ReaderSessionData {
    write_socket: Option<WriteHalf<TcpStream>>,
    name: Option<String>,
    ip: String,
    tables: HashMap<String, DateTimeAsMicroseconds>,
}

impl ReaderSessionData {
    pub fn new(write_socket: WriteHalf<TcpStream>, ip: String) -> Self {
        Self {
            write_socket: Some(write_socket),
            name: None,
            ip,
            tables: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> String {
        match &self.name {
            Some(name) => name.to_string(),
            None => self.ip.to_string(),
        }
    }

    async fn disconnect(&mut self) {
        let mut stream = None;
        std::mem::swap(&mut self.write_socket, &mut stream);

        let stream = stream.as_mut().unwrap();

        let result = stream.shutdown().await;

        if let Err(err) = result {
            println!(
                "Can not disconnect socket {}. Reason: {:?}",
                self.get_name(),
                err
            );
        }
    }

    pub fn get_tables(&self) -> Vec<String> {
        return self
            .tables
            .keys()
            .map(|table_name| table_name.to_string())
            .collect();
    }
}

pub struct ReaderSession {
    pub id: u64,
    pub ip: String,
    pub metrics: SessionMetrics,
    data: Mutex<ReaderSessionData>,
    pub logs: Arc<Logs>,
    pub last_incoming_package: AtomicDateTimeAsMicroseconds,
}

impl ReaderSession {
    pub fn new(id: u64, ip: String, write_socket: WriteHalf<TcpStream>, logs: Arc<Logs>) -> Self {
        Self {
            id,
            ip: ip.to_string(),
            data: Mutex::new(ReaderSessionData::new(write_socket, ip.to_string())),
            logs,
            metrics: SessionMetrics::new(id, ip),
            last_incoming_package: AtomicDateTimeAsMicroseconds::now(),
        }
    }

    pub async fn set_name(&self, app_name: String) {
        self.metrics.update_name(app_name.to_string()).await;
        let mut write_access = self.data.lock().await;
        write_access.name = Some(app_name);
    }

    pub async fn get_name(&self) -> String {
        let read_access = self.data.lock().await;
        return read_access.get_name();
    }

    pub async fn send_package(&self, contract: &TcpContract) -> Result<(), SendPackageError> {
        let bytes = contract.serialize();
        let mut write_access = self.data.lock().await;

        if let Some(socket) = &mut write_access.write_socket {
            let result = socket.write_all(bytes.as_ref()).await;

            if let Err(err) = result {
                self.logs
                    .add_error(
                        None,
                        app::logs::SystemProcess::TcpSocket,
                        format!("Send Package to socket:  {}", write_access.get_name()),
                        "Can not send to socket".to_string(),
                        Some(format!("{:?}", err)),
                    )
                    .await;

                write_access.disconnect().await;

                return Err(SendPackageError::JustDisconnected);
            } else {
                return Ok(());
            }
        } else {
            return Err(SendPackageError::Disconnected);
        }
    }

    pub async fn disconnect(&self) {
        let mut write_access = self.data.lock().await;
        write_access.disconnect().await;
    }

    pub async fn get_tables(&self) -> Vec<String> {
        let read_access = self.data.lock().await;
        read_access.get_tables()
    }

    pub async fn subscribe(&self, table_name: String) {
        let mut write_access = self.data.lock().await;

        write_access
            .tables
            .insert(table_name, DateTimeAsMicroseconds::now());
    }

    pub async fn has_table(&self, table_name: &str) -> bool {
        let read_access = self.data.lock().await;
        return read_access.tables.contains_key(table_name);
    }
}
