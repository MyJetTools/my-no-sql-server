use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use rust_extensions::events_loop::EventsLoop;
use tokio::sync::Mutex;

use crate::tcp::MyNoSqlTcpConnection;

use super::TcpPayloads;

pub struct TcpConnectionInfo {
    pub connection: Arc<MyNoSqlTcpConnection>,
    send_timeout: Duration,
    payloads_to_send: Mutex<TcpPayloads>,
    pending_to_send: AtomicUsize,
    pub flush_events_loop: EventsLoop<()>,
}

impl TcpConnectionInfo {
    pub fn new(
        connection: Arc<MyNoSqlTcpConnection>,
        flush_events_loop: EventsLoop<()>,
        send_timeout: Duration,
        max_payload: usize,
    ) -> Self {
        Self {
            connection,
            send_timeout,
            payloads_to_send: Mutex::new(TcpPayloads::new(max_payload)),
            flush_events_loop,
            pending_to_send: AtomicUsize::new(0),
        }
    }

    pub fn get_id(&self) -> i32 {
        self.connection.id
    }

    pub fn get_ip(&self) -> String {
        match &self.connection.addr {
            Some(addr) => format!("{}", addr),
            None => "unknown".to_string(),
        }
    }

    async fn get_next_payload(&self) -> Option<Vec<u8>> {
        let mut payloads_to_send = self.payloads_to_send.lock().await;
        payloads_to_send.get_payload()
    }

    pub async fn flush_payloads(&self) {
        while let Some(payload_to_send) = self.get_next_payload().await {
            let send_result = tokio::time::timeout(
                self.send_timeout,
                self.connection.send_bytes(payload_to_send.as_slice()),
            )
            .await;

            if let Err(_) = send_result {
                println!(
                    "Timeout while sending to connection {}",
                    self.connection.connection_name.get().await
                );

                self.connection.disconnect().await;
            }
        }
    }

    pub async fn send(&self, payload_to_send: &[u8]) {
        let mut payloads_to_send = self.payloads_to_send.lock().await;
        payloads_to_send.add_payload(payload_to_send);
        self.pending_to_send.store(
            payloads_to_send.get_size(),
            std::sync::atomic::Ordering::SeqCst,
        );
        self.flush_events_loop.send(());
    }

    pub fn get_pending_to_send(&self) -> usize {
        self.pending_to_send
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
