use std::sync::Arc;

use rust_extensions::events_loop::EventsLoopTick;

use crate::data_readers::tcp_connection::TcpConnectionInfo;

pub struct TcpConnectionSendEventLoop {
    tcp_connection_info: Arc<TcpConnectionInfo>,
}

impl TcpConnectionSendEventLoop {
    pub fn new(tcp_connection_info: Arc<TcpConnectionInfo>) -> Self {
        Self {
            tcp_connection_info,
        }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<()> for TcpConnectionSendEventLoop {
    async fn tick(&self, _model: ()) {
        self.tcp_connection_info.flush_payloads().await;
    }
}
