use std::sync::Arc;

use super::{http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(Arc<TcpConnectionInfo>),
    Http(HttpConnectionInfo),
}

impl DataReaderConnection {
    pub fn get_pending_to_send(&self) -> usize {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_pending_to_send(),
            DataReaderConnection::Http(http_info) => http_info.get_pending_to_send(),
        }
    }

    pub async fn get_name(&self) -> Option<String> {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_name().await,
            DataReaderConnection::Http(http_info) => http_info.get_name().await,
        }
    }

    pub async fn set_name(&self, name: String) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.set_name(name).await,
            DataReaderConnection::Http(http_info) => http_info.set_name(name).await,
        }
    }
}
