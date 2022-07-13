use std::sync::{atomic::AtomicUsize, Arc};

use tokio::sync::Mutex;

use crate::tcp::MyNoSqlTcpConnection;

use super::SendPerSecond;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
    pub name: Mutex<Option<String>>,
    sent_per_second_accumulator: AtomicUsize,
    pub sent_per_second: SendPerSecond,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self {
            connection,
            name: Mutex::new(None),
            sent_per_second_accumulator: AtomicUsize::new(0),
            sent_per_second: SendPerSecond::new(),
        }
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

    pub async fn get_name(&self) -> Option<String> {
        let read_access = self.name.lock().await;
        read_access.clone()
    }

    pub async fn set_name(&self, name: String) {
        let mut write_access = self.name.lock().await;
        *write_access = Some(name);
    }

    pub async fn send(&self, payload_to_send: &[u8]) {
        self.sent_per_second_accumulator
            .fetch_add(payload_to_send.len(), std::sync::atomic::Ordering::SeqCst);
        self.connection.send_bytes(payload_to_send).await;
    }

    pub async fn timer_1sec_tick(&self) {
        let value = self
            .sent_per_second_accumulator
            .swap(0, std::sync::atomic::Ordering::SeqCst);
        self.sent_per_second.add(value).await;
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.connection
            .statistics
            .pending_to_send_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
