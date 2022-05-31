use std::{sync::Arc, time::Duration};

use crate::tcp::MyNoSqlTcpConnection;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
    send_timeout: Duration,
}

impl TcpConnectionInfo {
    pub fn new(connection: Arc<MyNoSqlTcpConnection>) -> Self {
        Self {
            connection,
            send_timeout: Duration::from_secs(3),
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

    pub async fn send(&self, payload_to_send: &[u8]) {
        let send_result = tokio::time::timeout(
            self.send_timeout,
            self.connection.send_bytes(payload_to_send),
        )
        .await;

        if let Err(_) = send_result {
            println!(
                "Timeout while sending to connection {}",
                self.connection.connection_name.get().await
            );

            self.connection.disconnect().await;
        }
    }
}
