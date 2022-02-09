use std::sync::Arc;

use crate::tcp::MyNoSqlTcpConnection;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self { connection }
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

    pub async fn send(&self, payload_to_send: &[u8]) {
        self.connection.send_bytes(payload_to_send).await;
    }
}
