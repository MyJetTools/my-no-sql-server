use std::sync::Arc;

use my_azure_storage_sdk::AzureConnection;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    data_readers::{data_readers::DataReaders, data_readers_broadcast::DataReadersCommand},
    db::DbInstance,
    db_transactions::TransactionEvent,
    persistence::QueueToPersist,
    settings_reader::SettingsModel,
};

use super::{logs::Logs, metrics::PrometheusMetrics};

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub struct AppServices {
    pub db: DbInstance,
    pub queue_to_persist: QueueToPersist,
    pub logs: Logs,
    pub settings: SettingsModel,
    pub data_readers: DataReaders,

    pub data_readers_sender: UnboundedSender<DataReadersCommand>,

    pub metrics: PrometheusMetrics,
}

impl AppServices {
    pub fn new(
        settings: SettingsModel,
        data_readers_sender: UnboundedSender<DataReadersCommand>,
    ) -> Self {
        AppServices {
            settings,
            db: DbInstance::new(),
            queue_to_persist: QueueToPersist::new(),
            logs: Logs::new(),
            data_readers: DataReaders::new(),
            data_readers_sender,
            metrics: PrometheusMetrics::new(),
        }
    }

    pub async fn dispatch_event(&self, event: TransactionEvent) {
        let event = Arc::new(event);
        self.queue_to_persist.enqueue(event.clone()).await;

        self.post_command_to_data_readers(DataReadersCommand::TransactionEvent(event))
            .await;
    }

    pub async fn post_command_to_data_readers(&self, command: DataReadersCommand) {
        let result = self.data_readers_sender.send(command);

        if let Err(err) = result {
            self.logs
                .add_error(
                    None,
                    crate::app::logs::SystemProcess::System,
                    "post_data_readers_command".to_string(),
                    "Failed posting command".to_string(),
                    Some(format!("{}", err)),
                )
                .await;
        }
    }

    pub fn get_azure_connection(&self) -> Option<Arc<AzureConnection>> {
        if !self.settings.persist_to_blob() {
            return None;
        }
        let result = AzureConnection::from_conn_string(self.settings.persistence_dest.as_str());
        return Some(Arc::new(result));
    }
}
