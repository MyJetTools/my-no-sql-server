use hyper::Uri;
use std::time::Duration;

use super::TelemetryEvent;

pub struct TelemetryWriter {
    pub publisher: Option<tokio::sync::mpsc::UnboundedSender<TelemetryEvent>>,
}

impl TelemetryWriter {
    pub fn new() -> Self {
        Self { publisher: None }
    }

    pub fn get_telemetry_reader(&mut self) -> tokio::sync::mpsc::UnboundedReceiver<TelemetryEvent> {
        let (transactions_sender, transactions_receiver) = tokio::sync::mpsc::unbounded_channel();

        self.publisher = Some(transactions_sender);
        transactions_receiver
    }

    pub fn write_http_request_duration(
        &self,
        url: Uri,
        method: hyper::Method,
        status_code: u16,
        duration: Duration,
    ) {
        if let Some(publisher) = &self.publisher {
            let result = publisher.send(TelemetryEvent::HttpServerEvent {
                url,
                status_code,
                duration,
                method,
            });

            if let Err(err) = result {
                println!("Can not send telemetry event: {}", err)
            }
        }
    }
}
