use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(TcpConnectionInfo),
    Http(HttpConnectionInfo),
}

impl DataReaderConnection {
    pub fn connected(&self) -> DateTimeAsMicroseconds {
        match self {
            DataReaderConnection::Tcp(info) => info.connection.statistics.connected,
            DataReaderConnection::Http(info) => info.connected,
        }
    }
}
