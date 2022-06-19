use std::{sync::Arc, time::Duration};

use rust_extensions::{date_time::DateTimeAsMicroseconds, events_loop::EventsLoop};
use tokio::sync::RwLock;

use crate::{
    background::tcp_connection_send_event_loop::TcpConnectionSendEventLoop,
    tcp::MyNoSqlTcpConnection,
};

use super::{
    http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo, DataReader,
    DataReaderConnection, DataReadersData,
};

pub struct DataReadersList {
    data: RwLock<DataReadersData>,
    http_session_time_out: Duration,
    tcp_send_time_out: Duration,
}

impl DataReadersList {
    pub fn new(http_session_time_out: Duration, tcp_send_time_out: Duration) -> Self {
        Self {
            data: RwLock::new(DataReadersData::new()),
            http_session_time_out,
            tcp_send_time_out,
        }
    }

    pub async fn add_tcp(&self, tcp_connection: Arc<MyNoSqlTcpConnection>) {
        let id = format!("Tcp-{}", tcp_connection.id);
        println!("New tcp reader connnected {}", id);

        let flash_events_loop = EventsLoop::new(id.to_string());

        let connection_info = TcpConnectionInfo::new(
            tcp_connection,
            flash_events_loop,
            self.tcp_send_time_out,
            1024 * 1024 * 4,
        );

        let connection_info = Arc::new(connection_info);

        connection_info
            .flush_events_loop
            .register_event_loop(Arc::new(TcpConnectionSendEventLoop::new(
                connection_info.clone(),
            )))
            .await;

        let mut write_lock = self.data.write().await;

        let data_reader = DataReader::new(id, DataReaderConnection::Tcp(connection_info));
        write_lock.insert(Arc::new(data_reader));
    }

    pub async fn add_http(&self, ip: String) -> Arc<DataReader> {
        let mut write_lock = self.data.write().await;
        let id = format!("Http-{}", write_lock.get_next_id());

        let http_connection_info = HttpConnectionInfo::new(id.to_string(), ip);

        let data_reader = Arc::new(DataReader::new(
            id.clone(),
            DataReaderConnection::Http(http_connection_info),
        ));

        write_lock.insert(data_reader.clone());

        data_reader
    }

    pub async fn get_tcp(&self, tcp_connection: &MyNoSqlTcpConnection) -> Option<Arc<DataReader>> {
        let read_lock = self.data.read().await;
        read_lock.get_tcp(tcp_connection.id)
    }

    pub async fn get_http(&self, session_id: &str) -> Option<Arc<DataReader>> {
        let read_lock = self.data.read().await;
        read_lock.get_http(session_id)
    }

    pub async fn remove_tcp(
        &self,
        tcp_connection: &MyNoSqlTcpConnection,
    ) -> Option<Arc<DataReader>> {
        println!("Tcp reader is disconnnected {}", tcp_connection.id);
        let mut write_lock = self.data.write().await;
        write_lock.remove_tcp(tcp_connection.id)
    }

    pub async fn get_all(&self) -> Vec<Arc<DataReader>> {
        let read_lock = self.data.read().await;
        read_lock.get_all()
    }

    pub async fn get_subscribed_to_table(&self, table_name: &str) -> Option<Vec<Arc<DataReader>>> {
        let read_access = self.data.read().await;
        read_access.get_subscribred_to_table(table_name).await
    }

    pub async fn ten_seconds_tick(&self, now: DateTimeAsMicroseconds) {
        let mut write_access = self.data.write().await;
        write_access.gc_http_sessions(now, self.http_session_time_out);
    }
}
