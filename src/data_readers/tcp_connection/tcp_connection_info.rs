use std::sync::{atomic::AtomicUsize, Arc};

use my_no_sql_sdk::tcp_contracts::MyNoSqlTcpContract;
use my_tcp_sockets::tcp_connection::ConnectionStatistics;

use crate::tcp::MyNoSqlTcpConnection;

use super::SendPerSecond;

pub enum ReaderName {
    AsReader(String),
    AsNode { location: String, version: String },
}

impl ReaderName {
    pub fn is_node(&self) -> bool {
        match self {
            ReaderName::AsReader(_) => false,
            ReaderName::AsNode { .. } => true,
        }
    }

    pub fn get_name(&self) -> &str {
        match self {
            ReaderName::AsReader(name) => name,
            ReaderName::AsNode { location, .. } => location,
        }
    }
}

pub struct TcpConnectionInfo {
    connection: Arc<MyNoSqlTcpConnection>,
    pub name: ReaderName,
    sent_per_second_accumulator: AtomicUsize,
    pub sent_per_second: SendPerSecond,
    pub compress_data: bool,
}

impl TcpConnectionInfo {
    pub fn new(
        connection: Arc<MyNoSqlTcpConnection>,
        name: ReaderName,
        compress_data: bool,
    ) -> Self {
        Self {
            connection,
            name,
            sent_per_second_accumulator: AtomicUsize::new(0),
            sent_per_second: SendPerSecond::new(),
            compress_data,
        }
    }

    pub fn connection_statistics(&self) -> &ConnectionStatistics {
        self.connection.statistics()
    }

    pub fn is_node(&self) -> bool {
        self.name.is_node()
    }

    pub fn get_id(&self) -> i32 {
        self.connection.id
    }

    pub fn get_ip(&self) -> String {
        match &self.connection.addr {
            Some(addr) => format!("{}", addr),
            None => "unknown".to_string(),
        }
    }

    pub fn is_compressed_data(&self) -> bool {
        self.compress_data
    }

    pub fn get_name(&self) -> &str {
        self.name.get_name()
    }

    /*
    pub async fn set_name_as_reader(&self, name: String) {
        let mut write_access = self.name.lock().await;
        *write_access = ReaderName::AsReader(name);
    }

    pub async fn set_name_as_node(&self, location: String, version: String) {
        self.is_node.store(true, Ordering::SeqCst);
        let mut write_access = self.name.lock().await;
        *write_access = ReaderName::AsNode { location, version };
    }
     */

    pub async fn send(&self, tcp_contract: &[MyNoSqlTcpContract]) {
        let sent_amount = self.connection.send_many(tcp_contract).await;
        self.sent_per_second_accumulator
            .fetch_add(sent_amount, std::sync::atomic::Ordering::SeqCst);
    }

    pub async fn timer_1sec_tick(&self) {
        let value = self
            .sent_per_second_accumulator
            .swap(0, std::sync::atomic::Ordering::SeqCst);
        self.sent_per_second.add(value).await;
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.connection
            .statistics()
            .pending_to_send_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
