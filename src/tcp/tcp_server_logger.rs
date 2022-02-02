use std::sync::Arc;

use my_logger::MyLoggerReader;

use crate::app::logs::{Logs, SystemProcess};

pub struct TcpServerLogger {
    logger: Arc<Logs>,
}

impl TcpServerLogger {
    pub fn new(logger: Arc<Logs>) -> Self {
        Self { logger }
    }
}

impl MyLoggerReader for TcpServerLogger {
    fn write_log(&self, log_event: my_logger::MyLogEvent) {
        tokio::spawn(write_log(self.logger.clone(), log_event));
    }
}

async fn write_log(logger: Arc<Logs>, log_event: my_logger::MyLogEvent) {
    match log_event.level {
        my_logger::LogLevel::Info => {
            logger
                .add_info(
                    None,
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                )
                .await;
        }
        my_logger::LogLevel::Error => {
            logger
                .add_error(
                    None,
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                    log_event.context,
                )
                .await;
        }
        my_logger::LogLevel::FatalError => {
            logger
                .add_fatal_error(
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                )
                .await;
        }
    }
}
