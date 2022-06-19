use std::sync::Arc;

use super::{http_connection::HttpConnectionInfo, tcp_connection::TcpConnectionInfo};

pub enum DataReaderConnection {
    Tcp(Arc<TcpConnectionInfo>),
    Http(HttpConnectionInfo),
}
