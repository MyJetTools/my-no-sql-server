use super::{http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(TcpConnectionInfo),
    Http(HttpConnectionInfo),
}
