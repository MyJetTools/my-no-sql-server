use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::RwLock;

use crate::db::DbTable;

use super::{tcp_connection::TcpConnectionInfo, DataReaderUpdatableData};

pub struct DataReadeMetrics {
    pub session_id: String,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_moment: DateTimeAsMicroseconds,
    pub ip: String,
    pub name: Option<String>,
    pub tables: Vec<String>,
}

pub enum DataReaderConnection {
    Tcp(TcpConnectionInfo),
}

pub struct DataReader {
    pub id: String,
    data: RwLock<DataReaderUpdatableData>,
    pub connection: DataReaderConnection,
}

impl DataReader {
    pub fn new(id: String, connection: DataReaderConnection) -> Self {
        Self {
            id,
            data: RwLock::new(DataReaderUpdatableData::new()),
            connection,
        }
    }

    pub async fn has_table(&self, table_name: &str) -> bool {
        let read_access = self.data.read().await;
        read_access.has_table(table_name)
    }

    pub async fn set_name(&self, name: String) {
        let mut write_access = self.data.write().await;
        write_access.update_name(name);
    }

    pub async fn get_name(&self) -> Option<String> {
        let read_access = self.data.read().await;
        read_access.name.clone()
    }

    pub async fn subscribe(&self, db_table: Arc<DbTable>) {
        let mut write_access = self.data.write().await;
        write_access.subscribe(db_table);
    }

    fn get_ip(&self) -> String {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.get_ip(),
        }
    }

    fn get_connected_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection.connection.statistics.connected,
        }
    }

    fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        match &self.connection {
            DataReaderConnection::Tcp(connection) => connection
                .connection
                .statistics
                .last_receive_moment
                .as_date_time(),
        }
    }

    pub async fn get_metrics(&self) -> DataReadeMetrics {
        let session_id = self.id.to_string();
        let ip = self.get_ip();
        let connected = self.get_connected_moment();
        let last_incoming_moment = self.get_last_incoming_moment();

        let read_access = self.data.read().await;

        DataReadeMetrics {
            session_id,
            connected,
            last_incoming_moment,
            ip,
            name: read_access.name.clone(),
            tables: read_access.get_table_names(),
        }
    }
}
