use my_http_server::HttpFailResult;
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};
use tokio::sync::Mutex;

use crate::{
    data_readers::http_connection::connection_delivery_info::HttpPayload, db_sync::SyncEvent,
};

use super::{into_http_payload, HttpConnectionDeliveryInfo};

pub struct HttpConnectionInfo {
    pub id: String,
    pub ip: String,
    pub connected: DateTimeAsMicroseconds,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
    pub delivery_info: Mutex<HttpConnectionDeliveryInfo>,
}

impl HttpConnectionInfo {
    pub fn new(id: String, ip: String) -> Self {
        Self {
            id,
            ip,
            connected: DateTimeAsMicroseconds::now(),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
            delivery_info: Mutex::new(HttpConnectionDeliveryInfo::new()),
        }
    }

    pub async fn ping(&self, now: DateTimeAsMicroseconds) {
        let mut delivery_info = self.delivery_info.lock().await;
        delivery_info.ping(now)
    }

    pub async fn send(&self, sync_event: &SyncEvent) {
        if let Some(payload) = into_http_payload::convert(sync_event).await {
            let mut write_access = self.delivery_info.lock().await;
            write_access.upload(payload);

            if let Some(mut task) = write_access.get_task_to_write_response() {
                let payload = write_access.get_payload_to_deliver().unwrap();
                task.set_ok(HttpPayload::Payload(payload));
            }
        }
    }

    pub async fn new_request(&self) -> Result<HttpPayload, HttpFailResult> {
        let task_completion = {
            let mut write_access = self.delivery_info.lock().await;

            if let Some(payload) = write_access.get_payload_to_deliver() {
                return Ok(HttpPayload::Payload(payload));
            }

            write_access.issue_task_completion()
        };

        task_completion.get_result().await
    }
}
