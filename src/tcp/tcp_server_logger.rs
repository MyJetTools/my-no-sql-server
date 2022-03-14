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
        match log_event.level {
            my_logger::LogLevel::Info => {
                self.logger.add_info(
                    None,
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                );
            }
            my_logger::LogLevel::Error => {
                self.logger.add_error(
                    None,
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                    log_event.context,
                );
            }
            my_logger::LogLevel::FatalError => {
                self.logger.add_fatal_error(
                    SystemProcess::TcpSocket,
                    log_event.process,
                    log_event.message,
                );
            }
        }
    }
}
