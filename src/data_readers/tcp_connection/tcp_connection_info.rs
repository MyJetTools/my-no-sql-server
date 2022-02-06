use std::sync::Arc;

use crate::tcp::MyNoSqlTcpConnection;

use super::TcpPayloadToSend;

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

    pub async fn send(&self, payload_to_send: &TcpPayloadToSend) {
        match payload_to_send {
            TcpPayloadToSend::Single(payload) => {
                self.connection.send_bytes(payload).await;
            }
            TcpPayloadToSend::Multiple(payloads) => {
                for payload in payloads {
                    self.connection.send_bytes(payload).await;
                }
            }
            TcpPayloadToSend::FirstInit(tcp_contract) => {
                self.connection.send_ref(tcp_contract).await;
            }
        }
    }
}
