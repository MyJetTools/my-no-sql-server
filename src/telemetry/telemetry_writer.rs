use hyper::Uri;
use my_telemetry::MyTelemetry;
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

    pub fn write_dependency_request_duration(
        &self,
        host: String,
        protocol: String,
        resource: String,
        success: bool,
        duration: Duration,
    ) {
        if let Some(publisher) = &self.publisher {
            let result = publisher.send(TelemetryEvent::HttpDependencyEvent {
                host,
                protocol,
                resource,
                duration,
                success,
            });

            if let Err(err) = result {
                println!("Can not send telemetry event: {}", err)
            }
        }
    }
}

impl MyTelemetry for TelemetryWriter {
    fn track_url_duration(
        &self,
        method: hyper::Method,
        uri: hyper::Uri,
        http_code: u16,
        duration: Duration,
    ) {
        self.write_http_request_duration(uri, method, http_code, duration);
    }

    fn track_dependency_duration(
        &self,
        host: String,
        protocol: String,
        resource: String,
        success: bool,
        duration: Duration,
    ) {
        self.write_dependency_request_duration(host, protocol, resource, success, duration);
    }
}
