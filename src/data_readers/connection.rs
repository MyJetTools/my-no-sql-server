use std::sync::Arc;

use super::{http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(Arc<TcpConnectionInfo>),
    Http(HttpConnectionInfo),
}

impl DataReaderConnection {
    pub async fn get_name(&self) -> Option<String> {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_name().await,
            DataReaderConnection::Http(http_info) => http_info.get_name().await,
        }
    }

    pub async fn set_name_as_reader(&self, name: String) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.set_name_as_reader(name).await,
            DataReaderConnection::Http(http_info) => http_info.set_name_as_reader(name).await,
        }
    }

    pub async fn set_name_as_node(&self, location: String, version: String) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => {
                tcp_info.set_name_as_node(location, version).await
            }
            DataReaderConnection::Http(_) => {
                panic!("Node does not exist in HTTP Mode")
            }
        }
    }

    pub async fn one_sec_tick(&self) {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.timer_1sec_tick().await,
            DataReaderConnection::Http(_) => {}
        }
    }
}

#[async_trait::async_trait]
impl crate::app::UpdatePendingToSyncModel for DataReaderConnection {
    async fn get_name(&self) -> Option<String> {
        self.get_name().await
    }

    fn get_pending_to_sync(&self) -> usize {
        match self {
            DataReaderConnection::Tcp(tcp_info) => tcp_info.get_pending_to_send(),
            DataReaderConnection::Http(http_info) => http_info.get_pending_to_send(),
        }
    }
}
