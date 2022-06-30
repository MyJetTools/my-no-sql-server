use std::sync::Arc;

use tokio::sync::Mutex;

use crate::tcp::MyNoSqlTcpConnection;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,

    pub name: Mutex<Option<String>>,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self {
            connection,
            name: Mutex::new(None),
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
        self.connection.send_bytes(payload_to_send).await;
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.connection
            .statistics
            .pending_to_send_buffer_size
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
