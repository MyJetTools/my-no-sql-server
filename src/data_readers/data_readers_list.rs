use std::sync::Arc;

use tokio::sync::RwLock;

use crate::tcp::MyNoSqlTcpConnection;

use super::{tcp_connection::TcpConnectionInfo, DataReader, DataReaderConnection, DataReadersData};

pub struct DataReadersList {
    data: RwLock<DataReadersData>,
}

impl DataReadersList {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(DataReadersData::new()),
        }
    }

    pub async fn add_tcp(&self, tcp_connection: Arc<MyNoSqlTcpConnection>) {
        let id = format!("Tcp-{}", tcp_connection.id);
        let connection_info = TcpConnectionInfo::new(tcp_connection);
        let mut write_lock = self.data.write().await;
        write_lock.insert(DataReader::new(
            id,
            DataReaderConnection::Tcp(connection_info),
        ));
    }

    pub async fn get_tcp(&self, tcp_connection: &MyNoSqlTcpConnection) -> Option<Arc<DataReader>> {
        let read_lock = self.data.read().await;
        read_lock.get_tcp(tcp_connection.id)
    }

    pub async fn remove_tcp(&self, tcp_connection: &MyNoSqlTcpConnection) {
        let mut write_lock = self.data.write().await;
        write_lock.remove_tcp(tcp_connection.id);
    }

    pub async fn get_all(&self) -> Vec<Arc<DataReader>> {
        let read_lock = self.data.read().await;
        read_lock.get_all()
    }

    pub async fn get_subscribed_to_table(&self, table_name: &str) -> Option<Vec<Arc<DataReader>>> {
        let read_access = self.data.read().await;
        read_access.get_subscribred_to_table(table_name).await
    }
}
