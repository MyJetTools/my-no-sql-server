use std::sync::Arc;

use rust_extensions::events_loop::EventsLoopTick;

use crate::{app::AppContext, data_readers::tcp_connection::TcpConnectionInfo};

pub struct TcpConnectionSendEventLoop {
    app: Arc<AppContext>,
    tcp_connection_info: Arc<TcpConnectionInfo>,
}

impl TcpConnectionSendEventLoop {
    pub fn new(app: Arc<AppContext>, tcp_connection_info: Arc<TcpConnectionInfo>) -> Self {
        Self {
            tcp_connection_info,
            app,
        }
    }
}

#[async_trait::async_trait]
impl EventsLoopTick<()> for TcpConnectionSendEventLoop {
    async fn tick(&self, _model: ()) {
        self.tcp_connection_info.flush_payloads().await;
        self.app
            .metrics
            .update_pending_to_sync(self.tcp_connection_info.as_ref())
            .await;
    }
}
